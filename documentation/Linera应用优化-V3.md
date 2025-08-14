# contract.rs 结构优化逻辑与细节（第三版本，整合闭包多参数传递）

### contract.rs 结构优化逻辑与细节（第三版本，整合闭包多参数传递）

#### 一、核心优化目标与设计理念

**核心目标**：

在第二版本「分层职责架构」基础上，彻底移除全局 context（TLS），通过**闭包捕获依赖 + 泛型多参数适配**，实现依赖隐式传递、多参数灵活支持及 WASM 环境兼容。

**设计理念**：



*   保留第二版本的「合约入口→业务逻辑→接口→状态」分层架构，仅优化依赖与参数传递方式。

*   利用 Rust 闭包的环境捕获特性，将`state`和`runtime`作为闭包环境变量，避免全局存储。

*   通过泛型参数支持闭包传递任意数量参数（以 3 个参数为典型案例），兼顾灵活性与类型安全。

#### 二、整体架构分层（基于第二版本演进）



```
contract.rs（合约入口层）

├─ 核心职责：捕获依赖、构造带参数闭包、转发请求

├─ 关键逻辑：通过闭包捕获 \&mut state 和 \&runtime，传递多参数

└─ 对外接口：实现 Linera Contract trait

handlers/（业务逻辑层）

├─ 核心职责：接收带参数的闭包，执行业务逻辑

├─ 关键抽象：OperationHandler trait（支持泛型多参数）

└─ 实现示例：TransferHandler（2参数）、BatchTransferHandler（3参数）

interfaces/（抽象接口层）

├─ RuntimeContext trait（运行时能力抽象）

└─ StateInterface trait（状态操作抽象）（与第二版本一致）

state.rs（状态管理层）

└─ CreditState（实现 StateInterface）（与第二版本一致）
```

#### 三、各层详细设计与实现

##### 1. 合约入口层（contract.rs）：闭包捕获与参数传递

**核心逻辑**：

在`execute_operation`中，根据操作类型构造闭包（捕获`state`和`runtime`），并传递对应参数（以 3 个参数为典型），转发给 handlers 处理。



```
// src/contract.rs（第三版本）

impl Contract for CreditContract {

&#x20;   async fn execute\_operation(\&mut self, operation: Operation) -> Result<(), CreditError> {

&#x20;       // 捕获当前 state 和 runtime（闭包环境变量）

&#x20;       let state = \&mut self.state;

&#x20;       let runtime = \&self.runtime;

&#x20;       match operation {

&#x20;           // 案例1：传递2个参数（from/to/amount 实际为3个字段，此处简化为2个核心参数）

&#x20;           Operation::Transfer { from, to, amount } => {

&#x20;               TransferHandler::handle(

&#x20;                   move |f, t| async move { // 闭包接收2个参数（from/to）

&#x20;                       // 权限校验（使用闭包捕获的 runtime）

&#x20;                       let signer = runtime.authenticated\_signer()

&#x20;                           .ok\_or(CreditError::Unauthorized)?;

&#x20;                       if signer != f {

&#x20;                           return Err(CreditError::SignerMismatch);

&#x20;                       }

&#x20;                       // 执行转账（使用参数和捕获的 state）

&#x20;                       state.transfer(f, t, amount, runtime.system\_time()).await?;

&#x20;                       Ok(())

&#x20;                   },

&#x20;                   from, to // 传递2个参数

&#x20;               ).await

&#x20;           }

&#x20;           // 案例2：传递3个参数（核心优化案例）

&#x20;           Operation::BatchTransfer {&#x20;

&#x20;               from,&#x20;

&#x20;               to\_list,&#x20;

&#x20;               total\_amount&#x20;

&#x20;           } => {

&#x20;               BatchTransferHandler::handle(

&#x20;                   move |f, t\_list, t\_amount| async move { // 闭包接收3个参数

&#x20;                       // 1. 权限校验（使用捕获的 runtime）

&#x20;                       let signer = runtime.authenticated\_signer()

&#x20;                           .ok\_or(CreditError::Unauthorized)?;

&#x20;                       if signer != f {

&#x20;                           return Err(CreditError::SignerMismatch);

&#x20;                       }

&#x20;                       // 2. 校验总金额合理性（使用传递的3个参数）

&#x20;                       let per\_amount = t\_amount.checked\_div(t\_list.len() as u64)

&#x20;                           .ok\_or(CreditError::InvalidTotalAmount)?;

&#x20;                       // 3. 执行批量转账（使用参数和捕获的 state）

&#x20;                       for to in t\_list {

&#x20;                           state.transfer(

&#x20;                               f.clone(),

&#x20;                               to,

&#x20;                               per\_amount,

&#x20;                               runtime.system\_time()

&#x20;                           ).await?;

&#x20;                       }

&#x20;                       Ok(())

&#x20;                   },

&#x20;                   from, to\_list, total\_amount // 传递3个参数

&#x20;               ).await

&#x20;           }

&#x20;       }

&#x20;   }

}
```

**设计依据**：



*   闭包`move |params| async move { ... }`同时捕获`state`/`runtime`（环境变量）和`params`（显式参数），实现依赖与参数的统一传递。

*   按操作类型拆分处理逻辑，符合第二版本的「单一职责原则」，新增操作仅需新增`match`分支。

##### 2. 业务逻辑层（handlers）：泛型多参数适配

**核心抽象**：`OperationHandler` trait

支持接收带任意参数的闭包，通过泛型适配参数个数与类型：



```
// src/contract/handlers/mod.rs（第三版本）

pub trait OperationHandler {

&#x20;   type Output;

&#x20;   /// 接收带参数的闭包和对应参数，执行逻辑并返回结果

&#x20;   /// 泛型 A/B/C 对应参数类型，支持 1\~N 个参数（此处以3个为例扩展）

&#x20;   fn handle\<F, A, B, C>(logic: F, arg1: A, arg2: B, arg3: C) -> impl std::future::Future\<Output = Self::Output>

&#x20;   where

&#x20;       F: FnOnce(A, B, C) -> impl std::future::Future\<Output = Self::Output>,

&#x20;       A: Clone,

&#x20;       B: Clone,

&#x20;       C: Clone;

}
```

**处理器实现示例**：



*   **BatchTransferHandler（3 参数）**：



```
// src/contract/handlers/batch\_transfer.rs

pub struct BatchTransferHandler;

impl OperationHandler for BatchTransferHandler {

&#x20;   type Output = Result<(), CreditError>;

&#x20;   // 接收带3个参数的闭包，执行时传递参数

&#x20;   fn handle\<F, A, B, C>(logic: F, arg1: A, arg2: B, arg3: C) -> impl std::future::Future\<Output = Self::Output>

&#x20;   where

&#x20;       F: FnOnce(A, B, C) -> impl std::future::Future\<Output = Self::Output>,

&#x20;       A: Clone,

&#x20;       B: Clone,

&#x20;       C: Clone,

&#x20;   {

&#x20;       async move {

&#x20;           // 调用闭包时传递3个参数

&#x20;           logic(arg1, arg2, arg3).await

&#x20;       }

&#x20;   }

}
```



*   **TransferHandler（2 参数，兼容适配）**：

    对于参数个数较少的操作，可忽略多余泛型（或通过元组包装参数）：



```
// src/contract/handlers/transfer.rs

pub struct TransferHandler;

impl OperationHandler for TransferHandler {

&#x20;   type Output = Result<(), CreditError>;

&#x20;   // 接收带2个参数的闭包（第三参数用单元类型占位）

&#x20;   fn handle\<F, A, B, C>(logic: F, arg1: A, arg2: B, \_arg3: C) -> impl std::future::Future\<Output = Self::Output>

&#x20;   where

&#x20;       F: FnOnce(A, B) -> impl std::future::Future\<Output = Self::Output>,

&#x20;       A: Clone,

&#x20;       B: Clone,

&#x20;       C: Clone,

&#x20;   {

&#x20;       async move {

&#x20;           logic(arg1, arg2).await

&#x20;       }

&#x20;   }

}
```

**设计依据**：



*   泛型参数`A, B, C`无需知道具体类型，仅需满足闭包的参数要求，实现「参数透明化」。

*   处理器无需关心`state`和`runtime`的来源，仅通过闭包调用即可使用，符合「依赖倒置原则」。

##### 3. 接口层与状态层（复用第二版本）



*   **接口层**：`RuntimeContext`和`StateInterface`定义不变，确保 handlers 依赖抽象而非具体实现。



```
// src/contract/interfaces.rs（与第二版本一致）

pub trait RuntimeContext {

&#x20;   fn chain\_id(\&self) -> ChainId;

&#x20;   fn system\_time(\&self) -> Timestamp;

&#x20;   fn authenticated\_signer(\&self) -> Option\<AccountOwner>;

}

pub trait StateInterface {

&#x20;   async fn transfer(

&#x20;       \&mut self,

&#x20;       from: AccountOwner,

&#x20;       to: AccountOwner,

&#x20;       amount: Amount,

&#x20;       time: Timestamp

&#x20;   ) -> Result<(), CreditError>;

&#x20;  &#x20;

&#x20;   // 其他状态方法...

}
```



*   **状态层**：`CreditState`实现`StateInterface`，封装状态存储细节，与第二版本逻辑一致。

#### 四、测试方案：验证多参数传递与依赖捕获

以 3 参数的`BatchTransferHandler`为例，测试闭包参数传递和依赖使用的正确性：



```
\#\[cfg(test)]

mod tests {

&#x20;   use super::\*;

&#x20;   use crate::contract::mock::{MockState, MockRuntime};

&#x20;   \#\[tokio::test]

&#x20;   async fn test\_batch\_transfer\_with\_three\_params() {

&#x20;       // 1. 准备3个测试参数

&#x20;       let from = AccountOwner::default();

&#x20;       let to\_list = vec!\[AccountOwner::default(), AccountOwner::default()];

&#x20;       let total\_amount = Amount::from\_tokens(300); // 总金额300，分2个接收者

&#x20;       // 2. 准备Mock依赖（模拟 state 和 runtime）

&#x20;       let mut mock\_state = MockState::new();

&#x20;       let mut mock\_runtime = MockRuntime::new();

&#x20;       mock\_runtime.set\_authenticated\_signer(from.clone()); // 模拟签名者

&#x20;       // 3. 构造带3个参数的测试闭包（模拟合约入口层逻辑）

&#x20;       let test\_closure = move |f: AccountOwner, t\_list: Vec\<AccountOwner>, t\_amount: Amount| async move {

&#x20;           let per\_amount = t\_amount.checked\_div(t\_list.len() as u64).unwrap();

&#x20;           for to in t\_list {

&#x20;               mock\_state.transfer(f.clone(), to, per\_amount, mock\_runtime.system\_time()).await?;

&#x20;           }

&#x20;           Ok(())

&#x20;       };

&#x20;       // 4. 执行测试，验证参数传递和逻辑执行

&#x20;       let result = BatchTransferHandler::handle(test\_closure, from, to\_list, total\_amount).await;

&#x20;       assert!(result.is\_ok());

&#x20;       assert\_eq!(mock\_state.transfer\_count(), 2); // 验证两次转账

&#x20;       assert\_eq!(mock\_state.total\_transferred\_amount(), Amount::from\_tokens(300)); // 验证总金额

&#x20;   }

}
```

#### 五、第三版本 vs 第二版本：核心优势对比



| 维度       | 第二版本（全局 context）                    | 第三版本（闭包多参数）              |
| -------- | ----------------------------------- | ------------------------ |
| 依赖传递方式   | `Context::current()` 全局获取           | 闭包环境捕获，局部作用域内有效          |
| 参数支持     | 需在 context 中封装，扩展性差                 | 泛型支持任意参数个数，新增参数无需修改接口    |
| WASM 兼容性 | 依赖 TLS，可能存在隐患                       | 闭包原生支持，无额外依赖             |
| 代码侵入性    | handlers 需显式调用 `Context::current()` | handlers 仅关注业务逻辑，无依赖获取代码 |
| 安全性      | 全局状态可能存在生命周期漏洞                      | 依赖作用域受限，编译期检查生命周期        |

#### 六、总结

第三版本通过「闭包捕获依赖 + 泛型多参数适配」，在保留第二版本分层架构优势的同时，解决了全局 context 的安全性与灵活性问题：



1.  彻底移除全局状态，依赖仅在闭包作用域内有效，符合 WASM 环境要求。

2.  支持传递 2 个、3 个甚至更多参数，新增业务操作时无需修改接口层。

3.  handlers 代码更简洁，仅需关注业务逻辑，依赖与参数传递完全透明。

该方案兼顾了类型安全、灵活性与环境兼容性，是合约代码结构优化的最终演进形态。

> （注：文档部分内容可能由 AI 生成）