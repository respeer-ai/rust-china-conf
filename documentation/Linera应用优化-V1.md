# contract.rs 结构优化方案（最终版）

## 一、核心目标与整体架构设计

### 优化核心目标

将原 `contract.rs` 中耦合的「合约入口、业务逻辑、运行时交互、状态操作」拆分为**分层职责架构**，使复杂功能可管理、可扩展、可测试。

### 优化依据与原则



*   **单一职责原则（SRP）**：一个模块 / 结构体仅负责一项职责。拆分后每层仅处理一类任务，降低耦合风险。

*   **高内聚低耦合原则**：功能相关的代码聚合在同一层，不同层通过接口交互。例如业务逻辑层专注于业务规则，与运行时 / 状态的交互通过接口完成。

*   **复杂度管理需求**：分层架构将复杂度分散到各层，符合「分而治之」的工程思想，便于功能扩展（如新增信用分计算、多链同步）。

### 整体架构分层（自顶向下）



```
contract/（合约模块）

├── mod.rs（合约入口：组装与转发）

├── runtime.rs（运行时适配器）

├── types.rs（合约专属类型：Operation/Message）

├── errors.rs（合约错误定义）

└── handlers/（业务逻辑处理器，内嵌于contract）

   ├── mod.rs（处理器trait+工厂）

   ├── operation/（操作处理器）

   │   ├── transfer.rs

   │   └── reward.rs

   └── message/（消息处理器）

       ├── receive\_funds.rs

       └── sync\_credit.rs

service/（内部服务模块）

├── mod.rs（服务入口）

├── types.rs（服务专属类型）

├── errors.rs（服务错误定义）

└── credit\_score.rs（信用分计算逻辑）

state/（状态模块）

├── mod.rs（状态入口）

├── types.rs（状态相关类型）

├── errors.rs（状态错误定义）

├── models.rs（状态数据模型）

├── storage.rs（状态存储实现）

└── interfaces/（拆分的状态接口）

   ├── balance.rs（BalanceState接口）

   ├── reward.rs（RewardState接口）

   └── mod.rs（组合接口StateInterface）
```

## 二、第一层：合约入口层（contract 模块）

### 核心定位

作为合约对外的唯一入口，仅负责「请求转发、依赖组装、生命周期管理」，不包含具体业务逻辑。

### 优化细节

#### 1. 核心结构体与接口实现



```
// contract/mod.rs

use linera\_contract::Contract;

use linera\_sdk::types::ChainId;

use std::sync::Arc;

use crate::{

   contract::{

       errors::ContractError,

       handlers::{Handler, HandlerFactory},

       runtime::ContractRuntimeAdapter,

       types::{Message, Operation, Response},

   },

   state::{storage::CreditStateStorage, interfaces::StateInterface},

};

/// 合约核心结构体：仅负责组装依赖

pub struct CreditContract {

   state: CreditStateStorage,

   runtime: ContractRuntimeAdapter,

}

\#\[async\_trait]

impl Contract for CreditContract {

   type Operation = Operation;

   type Message = Message;

   type Response = Response;

   /// 加载合约状态与依赖

   async fn load(runtime: linera\_contract::ContractRuntime\<Self>) -> Self {

       let state = CreditStateStorage::load(runtime.root\_view\_storage\_context())

           .await

           .expect("状态加载失败");

       Self {

           state,

           runtime: ContractRuntimeAdapter::new(runtime),

       }

   }

   /// 处理链上操作：通过工厂模式分发到对应处理器

   async fn execute\_operation(\&mut self, operation: Operation) -> Result<(), ContractError> {

       let mut handler = HandlerFactory::create(\&operation, \&mut self.state, \&self.runtime)?;

       let result = handler.handle().await?;

       // 统一处理事件发送

       for event in result.events {

           self.runtime.send\_message(event, self.runtime.chain\_id())?;

       }

       Ok(())

   }

   /// 处理跨链消息：同理分发到消息处理器

   async fn execute\_message(\&mut self, message: Message) -> Result\<Response, ContractError> {

       // 实现逻辑与execute\_operation类似

       Ok(Response::Empty)

   }

}

// 导出子模块

pub mod handlers;

pub mod runtime;

pub mod types;

pub mod errors;
```

#### 2. 处理器工厂模式



```
// contract/handlers/mod.rs

use crate::{

   contract::{

       errors::HandlerError,

       runtime::RuntimeContext,

       types::{Operation, Message},

   },

   state::interfaces::StateInterface,

};

/// 处理器标准返回结果

\#\[derive(Debug, Default)]

pub struct HandlerResult {

   pub events: Vec\<Message>, // 跨链事件

   pub logs: Vec\<String>,    // 审计日志

}

/// 处理器返回值类型

pub type HandlerOutcome = Result\<HandlerResult, HandlerError>;

/// 处理器通用接口

\#\[async\_trait]

pub trait Handler {

   async fn handle(\&mut self) -> HandlerOutcome;

}

/// 处理器工厂：集中管理操作到处理器的映射

pub struct HandlerFactory;

impl HandlerFactory {

   /// 根据操作类型创建对应处理器

   pub fn create<'a>(

       op: \&Operation,

       state: &'a mut impl StateInterface,

       runtime: &'a impl RuntimeContext,

   ) -> Result\<Box\<dyn Handler + 'a>, HandlerError> {

       match op {

           Operation::Transfer { from, to, amount } => Ok(Box::new(

               operation::transfer::TransferHandler::new(

                   state,

                   runtime,

                   from.clone(),

                   to.clone(),

                   \*amount,

               ),

           )),

           Operation::Reward { owner, amount } => Ok(Box::new(

               operation::reward::RewardHandler::new(

                   state,

                   runtime,

                   owner.clone(),

                   \*amount,

               ),

           )),

       }

   }

}

// 导出操作和消息处理器子模块

pub mod operation;

pub mod message;
```

## 三、第二层：业务逻辑层（contract/handlers 模块）

### 核心定位

通过「处理器模式」封装单一业务逻辑，每个处理器仅负责一种操作（如转账、奖励），依赖抽象接口而非具体实现。

### 优化细节

#### 1. 操作处理器实现（转账示例）



```
// contract/handlers/operation/transfer.rs

use async\_trait::async\_trait;

use crate::{

   contract::{

       errors::HandlerError,

       handlers::{Handler, HandlerResult, HandlerOutcome},

       runtime::RuntimeContext,

       types::Message,

   },

   state::interfaces::{BalanceState, StateInterface},

};

/// 转账处理器：仅负责转账业务逻辑

pub struct TransferHandler<'a> {

   state: &'a mut dyn BalanceState, // 仅依赖Balance领域接口

   runtime: &'a dyn RuntimeContext,

   from: linera\_sdk::types::AccountOwner,

   to: linera\_sdk::types::AccountOwner,

   amount: linera\_sdk::types::Amount,

}

impl<'a> TransferHandler<'a> {

   pub fn new(

       state: &'a mut impl StateInterface, // 自动满足BalanceState

       runtime: &'a impl RuntimeContext,

       from: linera\_sdk::types::AccountOwner,

       to: linera\_sdk::types::AccountOwner,

       amount: linera\_sdk::types::Amount,

   ) -> Self {

       Self {

           state,

           runtime,

           from,

           to,

           amount,

       }

   }

}

\#\[async\_trait]

impl<'a> Handler for TransferHandler<'a> {

   async fn handle(\&mut self) -> HandlerOutcome {

       // 1. 权限校验（通过运行时接口）

       let signer = self.runtime.authenticated\_signer()

           .ok\_or(HandlerError::Unauthenticated)?;

       if signer != self.from {

           return Err(HandlerError::PermissionDenied);

       }

       // 2. 执行转账（通过状态接口）

       let now = self.runtime.system\_time();

       self.state.transfer(

           self.from.clone(),

           self.to.clone(),

           self.amount,

           now,

       ).await.map\_err(HandlerError::StateError)?;

       // 3. 返回标准化结果

       Ok(HandlerResult {

           events: vec!\[Message::TransferEvent {

               from: self.from.clone(),

               to: self.to.clone(),

               amount: self.amount,

           }],

           logs: vec!\[format!(

               "Transfer: {} -> {}: {}",

               self.from, self.to, self.amount

           )],

       })

   }

}
```

#### 2. 错误处理定义



```
// contract/errors.rs

use thiserror::Error;

use crate::state::errors::StateError;

use linera\_sdk::error::RuntimeError;

\#\[derive(Debug, Error)]

pub enum ContractError {

   \#\[error("处理器错误: {0}")]

   Handler(#\[from] HandlerError),

   \#\[error("运行时错误: {0}")]

   Runtime(#\[from] RuntimeError),

}

\#\[derive(Debug, Error)]

pub enum HandlerError {

   \#\[error("未认证的调用者")]

   Unauthenticated,

   \#\[error("权限不足")]

   PermissionDenied,

   \#\[error("状态操作错误: {0}")]

   StateError(#\[from] StateError),

   \#\[error("不支持的操作类型")]

   UnsupportedOperation,

}
```

## 四、第三层：抽象接口层

### 核心定位

通过拆分的状态接口和运行时接口，隔离业务逻辑与底层依赖，实现「依赖倒置」。

### 优化细节

#### 1. 状态接口拆分（按领域）



```
// state/interfaces/balance.rs

use async\_trait::async\_trait;

use crate::{

   state::types::{AccountOwner, Amount, Timestamp},

   state::errors::StateError,

};

/// 余额领域接口：仅包含余额相关操作

\#\[async\_trait]

pub trait BalanceState {

   async fn transfer(

       \&mut self,

       from: AccountOwner,

       to: AccountOwner,

       amount: Amount,

       time: Timestamp,

   ) -> Result<(), StateError>;

   async fn get\_balance(\&self, owner: \&AccountOwner) -> Result\<Amount, StateError>;

}
```



```
// state/interfaces/reward.rs

use async\_trait::async\_trait;

use crate::{

   state::types::{AccountOwner, Amount, Timestamp},

   state::errors::StateError,

};

/// 奖励领域接口：仅包含奖励相关操作

\#\[async\_trait]

pub trait RewardState {

   async fn reward(

       \&mut self,

       owner: AccountOwner,

       amount: Amount,

       time: Timestamp,

   ) -> Result<(), StateError>;

   async fn get\_reward\_history(\&self, owner: \&AccountOwner) -> Result\<Vec\<Amount>, StateError>;

}
```



```
// state/interfaces/mod.rs

//! 状态接口组合：通过继承小接口形成完整能力

pub use balance::BalanceState;

pub use reward::RewardState;

/// 完整状态接口 = 各领域接口的组合（自动实现）

pub trait StateInterface: BalanceState + RewardState {}

impl\<T> StateInterface for T where T: BalanceState + RewardState {}

mod balance;

mod reward;
```

#### 2. 运行时接口设计



```
// contract/runtime.rs

use async\_trait::async\_trait;

use linera\_sdk::{

   types::{AccountOwner, ChainId, Timestamp},

   ContractRuntime,

};

use crate::contract::types::Message;

/// 运行时能力抽象接口

\#\[async\_trait]

pub trait RuntimeContext {

   fn chain\_id(\&self) -> ChainId;

   fn system\_time(\&self) -> Timestamp;

   fn authenticated\_signer(\&self) -> Option\<AccountOwner>;

   fn send\_message(\&self, msg: Message, target: ChainId) -> Result<(), crate::contract::errors::HandlerError>;

}

/// 运行时适配器：实现RuntimeContext接口

pub struct ContractRuntimeAdapter {

   inner: ContractRuntime\<crate::contract::CreditContract>,

}

impl ContractRuntimeAdapter {

   pub fn new(inner: ContractRuntime\<crate::contract::CreditContract>) -> Self {

       Self { inner }

   }

}

\#\[async\_trait]

impl RuntimeContext for ContractRuntimeAdapter {

   fn chain\_id(\&self) -> ChainId {

       self.inner.chain\_id()

   }

   fn system\_time(\&self) -> Timestamp {

       self.inner.current\_time()

   }

   fn authenticated\_signer(\&self) -> Option\<AccountOwner> {

       self.inner.authenticated\_signer()

   }

   fn send\_message(\&self, msg: Message, target: ChainId) -> Result<(), crate::contract::errors::HandlerError> {

       self.inner.send\_message(msg, target)

           .map\_err(|e| crate::contract::errors::HandlerError::RuntimeError(e))

   }

}
```

## 五、第四层：状态管理层（state 模块）

### 核心定位

专注于「状态存储与读写」，通过实现拆分的状态接口对接业务逻辑，隐藏底层存储细节。

### 优化细节

#### 1. 状态数据模型



```
// state/models.rs

//! 状态数据模型：仅定义数据结构，不包含业务逻辑

use serde::{Deserialize, Serialize};

use linera\_sdk::views::{View, MapView};

use crate::state::types::AccountOwner;

/// 余额映射模型（基于Linera的MapView实现持久化）

\#\[derive(Debug, Serialize, Deserialize)]

pub struct BalanceMap {

   inner: MapView\<AccountOwner, u64>, // 实际存储结构

}

impl BalanceMap {

   // 基础CRUD方法

   pub async fn get(\&self, owner: \&AccountOwner) -> Option\<u64> {

       self.inner.get(owner).await.ok().flatten()

   }

   pub async fn set(\&mut self, owner: AccountOwner, amount: u64) -> Result<(), String> {

       self.inner.insert(owner, amount).await.map\_err(|e| e.to\_string())

   }

   pub async fn update\<F: FnOnce(Option\<u64>) -> Option\<u64>>(

       \&mut self,

       owner: AccountOwner,

       f: F,

   ) -> Result<(), String> {

       let current = self.get(\&owner).await;

       let new\_value = f(current);

       match new\_value {

           Some(v) => self.set(owner, v).await,

           None => self.inner.remove(\&owner).await.map\_err(|e| e.to\_string()),

       }

   }

}

// 实现Linera的View trait（用于持久化）

\#\[async\_trait]

impl View for BalanceMap {

   type Error = String;

   async fn load(context: \&linera\_sdk::views::ViewStorageContext) -> Result\<Self, Self::Error> {

       Ok(Self {

           inner: MapView::load("balances", context).await.map\_err(|e| e.to\_string())?,

       })

   }

   async fn save(\&mut self) -> Result<(), Self::Error> {

       self.inner.save().await.map\_err(|e| e.to\_string())

   }

}
```

#### 2. 状态存储实现



```
// state/storage.rs

use async\_trait::async\_trait;

use linera\_sdk::views::ViewStorageContext;

use crate::{

   state::{

       errors::StateError,

       models::BalanceMap,

       types::{AccountOwner, Amount, Timestamp},

       interfaces::{BalanceState, RewardState},

   },

};

/// 状态存储实现：实现所有状态接口

pub struct CreditStateStorage {

   balances: BalanceMap,

   // 其他状态模型（如奖励记录）

   storage\_context: ViewStorageContext,

}

impl CreditStateStorage {

   /// 加载状态（从链上存储读取）

   pub async fn load(context: ViewStorageContext) -> Result\<Self, StateError> {

       let balances = BalanceMap::load(\&context).await.map\_err(StateError::Storage)?;

       Ok(Self {

           balances,

           storage\_context: context,

       })

   }

   /// 事务化操作：确保状态一致性

   async fn with\_transaction\<F, R>(\&mut self, f: F) -> Result\<R, StateError>

   where

       F: FnOnce(\&mut Self) -> Result\<R, StateError>,

   {

       // 简化实现：实际项目中可添加快照回滚逻辑

       let result = f(self).await?;

       self.balances.save().await.map\_err(StateError::Storage)?;

       Ok(result)

   }

}

/// 实现余额领域接口

\#\[async\_trait]

impl BalanceState for CreditStateStorage {

   async fn transfer(\</doubaocanvas>
```

> （注：文档部分内容可能由 AI 生成）
