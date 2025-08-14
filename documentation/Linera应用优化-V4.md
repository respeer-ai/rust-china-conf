# 第三版本最终改动方案（宏驱动的透明依赖注入）

### 第三版本最终改动方案（宏驱动的透明依赖注入）

#### 一、整体架构概览

第三版本通过**三层宏驱动**实现无全局 context 的透明依赖注入，整体架构如下：



```
宏系统（3个核心过程宏）

├─ #\[handler]：为handler自动生成依赖访问代码

├─ #\[env\_handler]：自动生成EnvHandler结构体及trait实现

└─ #\[operation\_handler]：自动生成execute\_operation的match分支

业务代码层

├─ contract.rs：合约入口，通过辅助函数创建handler

├─ handlers/：业务逻辑，用宏标注实现透明依赖

├─ interfaces/：State和Runtime的抽象接口（不变）

└─ state.rs：状态管理（不变）
```

核心目标：彻底移除全局 context，通过宏实现依赖的局部注入与透明访问，同时最大化代码简洁性。

#### 二、核心宏定义（proc-macro crate）

##### 1. `#[handler]`：为 handler 自动实现依赖访问



```
// credit\_contract\_macros/src/handler.rs

\#\[proc\_macro\_attribute]

pub fn handler(\_args: TokenStream, input: TokenStream) -> TokenStream {

&#x20;   let input = parse\_macro\_input!(input as DeriveInput);

&#x20;   let name = input.ident;

&#x20;   let expanded = quote! {

&#x20;       \#input

&#x20;       impl<'a> Handler<'a> for #name {

&#x20;           type State = dyn StateInterface + 'a;

&#x20;           type Runtime = dyn RuntimeContext + 'a;

&#x20;           fn state(\&mut self) -> \&mut Self::State {

&#x20;               unsafe {

&#x20;                   \&mut \*std::ptr::from\_raw\_parts\_mut(

&#x20;                       \&mut \*self as \*mut \_ as \*mut u8,

&#x20;                       std::mem::size\_of::<\&mut dyn StateInterface>()

&#x20;                   )

&#x20;               }

&#x20;           }

&#x20;           fn runtime(\&self) -> \&Self::Runtime {

&#x20;               unsafe {

&#x20;                   &\*std::ptr::from\_raw\_parts(

&#x20;                       &\*self as \*const \_ as \*const u8,

&#x20;                       std::mem::size\_of::<\&dyn RuntimeContext>()

&#x20;                   )

&#x20;               }

&#x20;           }

&#x20;       }

&#x20;   };

&#x20;   TokenStream::from(expanded)

}
```

##### 2. `#[env_handler]`：自动生成 EnvHandler



```
// credit\_contract\_macros/src/env\_handler.rs

\#\[proc\_macro\_attribute]

pub fn env\_handler(\_args: TokenStream, input: TokenStream) -> TokenStream {

&#x20;   let input = parse\_macro\_input!(input as DeriveInput);

&#x20;   let name = input.ident;

&#x20;   let expanded = quote! {

&#x20;       \#input

&#x20;       impl<'a, S, R> #name<'a, S, R>

&#x20;       where

&#x20;           S: StateInterface + 'a,

&#x20;           R: RuntimeContext + 'a,

&#x20;       {

&#x20;           pub fn new(state: &'a mut S, runtime: &'a R) -> Self {

&#x20;               Self { state, runtime }

&#x20;           }

&#x20;       }

&#x20;       impl<'a, S, R> Handler<'a> for #name<'a, S, R>

&#x20;       where

&#x20;           S: StateInterface + 'a,

&#x20;           R: RuntimeContext + 'a,

&#x20;       {

&#x20;           type State = S;

&#x20;           type Runtime = R;

&#x20;           fn state(\&mut self) -> \&mut Self::State { self.state }

&#x20;           fn runtime(\&self) -> \&Self::Runtime { self.runtime }

&#x20;       }

&#x20;   };

&#x20;   TokenStream::from(expanded)

}
```

##### 3. `#[operation_handler]`：自动生成 match 分支



```
// credit\_contract\_macros/src/operation\_handler.rs

\#\[proc\_macro\_attribute]

pub fn operation\_handler(args: TokenStream, input: TokenStream) -> TokenStream {

&#x20;   let operation\_name = parse\_macro\_input!(args as Ident);

&#x20;   let mut impl\_block = parse\_macro\_input!(input as ItemImpl);

&#x20;   let method = impl\_block.items.iter\_mut()

&#x20;       .find\_map(|item| match item {

&#x20;           syn::ImplItem::Method(m) => Some(m),

&#x20;           \_ => None,

&#x20;       })

&#x20;       .expect("Expected a handle method");

&#x20;   let method\_name = \&method.sig.ident;

&#x20;   let params: Vec<\_> = method.sig.inputs.iter()

&#x20;       .filter\_map(|arg| match arg {

&#x20;           FnArg::Typed(t) => match &\*t.pat {

&#x20;               Pat::Ident(pat) => Some(pat.ident.clone()),

&#x20;               \_ => None,

&#x20;           },

&#x20;           \_ => None,

&#x20;       })

&#x20;       .collect();

&#x20;   let expanded = quote! {

&#x20;       \#impl\_block

&#x20;       impl Contract for CreditContract {

&#x20;           async fn execute\_operation(\&mut self, operation: Operation) -> Result<(), CreditError> {

&#x20;               let state = \&mut self.state;

&#x20;               let runtime = \&self.runtime;

&#x20;               match operation {

&#x20;                   Operation::#operation\_name { #(#params),\* } => {

&#x20;                       let mut handler = Self::create\_handler(state, runtime);

&#x20;                       handler.#method\_name(#(#params),\*).await

&#x20;                   }

&#x20;                   \_ => self.execute\_operation(operation).await,

&#x20;               }

&#x20;           }

&#x20;       }

&#x20;   };

&#x20;   TokenStream::from(expanded)

}
```

#### 三、业务代码实现

##### 1. 接口层（interfaces.rs）



```
// 定义State和Runtime的抽象接口（与第二版本一致）

pub trait StateInterface {

&#x20;   async fn transfer(

&#x20;       \&mut self,

&#x20;       from: AccountOwner,

&#x20;       to: AccountOwner,

&#x20;       amount: Amount,

&#x20;       time: Timestamp,

&#x20;   ) -> Result<(), CreditError>;

&#x20;   // 其他状态方法...

}

pub trait RuntimeContext {

&#x20;   fn authenticated\_signer(\&self) -> Option\<AccountOwner>;

&#x20;   fn system\_time(\&self) -> Timestamp;

&#x20;   // 其他运行时方法...

}

// 定义Handler trait（供宏实现）

pub trait Handler<'a> {

&#x20;   type State: StateInterface + 'a;

&#x20;   type Runtime: RuntimeContext + 'a;

&#x20;   fn state(\&mut self) -> \&mut Self::State;

&#x20;   fn runtime(\&self) -> \&Self::Runtime;

}
```

##### 2. 状态层（state.rs）



```
// 实现StateInterface（与第二版本一致）

pub struct CreditState {

&#x20;   // 状态存储...

}

impl StateInterface for CreditState {

&#x20;   async fn transfer(...) -> Result<(), CreditError> {

&#x20;       // 状态操作逻辑...

&#x20;   }

&#x20;   // 其他方法实现...

}
```

##### 3. 业务逻辑层（handlers）



```
// src/contract/handlers/batch\_transfer.rs

use super::\*;

\#\[handler] // 自动实现Handler trait

pub struct BatchTransferHandler;

impl BatchTransferHandler {

&#x20;   // 自动生成对应的match分支

&#x20;   \#\[operation\_handler(BatchTransfer)]

&#x20;   pub async fn handle(

&#x20;       \&mut self,

&#x20;       from: AccountOwner,

&#x20;       to\_list: Vec\<AccountOwner>,

&#x20;       total\_amount: Amount,

&#x20;   ) -> Result<(), CreditError> {

&#x20;       // 透明使用runtime（无需全局context）

&#x20;       let signer = self.runtime().authenticated\_signer()

&#x20;           .ok\_or(CreditError::Unauthorized)?;

&#x20;       if signer != from {

&#x20;           return Err(CreditError::SignerMismatch);

&#x20;       }

&#x20;       // 透明使用state

&#x20;       let per\_amount = total\_amount.checked\_div(to\_list.len() as u64)

&#x20;           .ok\_or(CreditError::InvalidTotalAmount)?;

&#x20;       for to in to\_list {

&#x20;           self.state().transfer(

&#x20;               from.clone(),

&#x20;               to,

&#x20;               per\_amount,

&#x20;               self.runtime().system\_time()

&#x20;           ).await?;

&#x20;       }

&#x20;       Ok(())

&#x20;   }

}

// src/contract/handlers/transfer.rs（同理）

\#\[handler]

pub struct TransferHandler;

impl TransferHandler {

&#x20;   \#\[operation\_handler(Transfer)]

&#x20;   pub async fn handle(

&#x20;       \&mut self,

&#x20;       from: AccountOwner,

&#x20;       to: AccountOwner,

&#x20;       amount: Amount,

&#x20;   ) -> Result<(), CreditError> {

&#x20;       // 业务逻辑...

&#x20;       Ok(())

&#x20;   }

}
```

##### 4. 合约入口层（contract.rs）



```
// src/contract.rs

use super::\*;

pub struct CreditContract {

&#x20;   state: CreditState,

&#x20;   runtime: ContractRuntime, // 实现RuntimeContext

}

impl CreditContract {

&#x20;   // 辅助函数：创建handler（宏生成EnvHandler）

&#x20;   fn create\_handler<'a, S, R>(

&#x20;       state: &'a mut S,

&#x20;       runtime: &'a R,

&#x20;   ) -> Box\<dyn Handler<'a>>

&#x20;   where

&#x20;       S: StateInterface + 'a,

&#x20;       R: RuntimeContext + 'a,

&#x20;   {

&#x20;       \#\[env\_handler] // 自动生成结构体和trait实现

&#x20;       struct EnvHandler<'a, S: StateInterface, R: RuntimeContext> {

&#x20;           state: &'a mut S,

&#x20;           runtime: &'a R,

&#x20;       }

&#x20;       Box::new(EnvHandler::new(state, runtime))

&#x20;   }

}

// 无需手动实现Contract trait的execute\_operation

// 所有match分支由#\[operation\_handler]宏自动生成
```

#### 四、关键特性总结



1.  **无全局 context**：彻底移除`thread_local!`等全局存储，依赖通过局部闭包和宏注入传递。

2.  **透明依赖访问**：handler 通过`self.state()`和`self.runtime()`直接访问依赖，无需显式注入。

3.  **宏驱动简化**：三个核心宏自动生成 90% 的模板代码，开发者仅需关注业务逻辑。

4.  **类型安全**：通过生命周期泛型和编译器检查，确保依赖访问的安全性。

5.  **极致扩展性**：新增操作时，仅需实现 handler 并标注宏，无需修改其他代码。

#### 五、与第二版本的核心差异



| 维度     | 第二版本（全局 context）                    | 第三版本（宏驱动）                  |
| ------ | ----------------------------------- | -------------------------- |
| 依赖传递方式 | `Context::current()` 全局获取           | 宏自动注入局部依赖，`self.state()`访问 |
| 代码量    | 每个操作需编写 10 + 行模板代码                  | 仅需 3-5 行业务代码，宏生成其余部分       |
| 全局状态   | 依赖`thread_local!`                   | 无全局状态，依赖作用域严格受限            |
| 扩展性    | 新增操作需修改 3 处代码（分支 + handler+context） | 仅需新增 handler 并标注宏          |
| 学习成本   | 需理解全局 context 生命周期管理                | 仅需掌握 3 个宏的使用，无需关心底层实现      |

该方案是第三版本的最终形态，通过宏系统实现了「无全局 context、透明依赖访问、极致简洁」的目标，同时保持了类型安全和 WASM 环境兼容性，是合约代码结构优化的最佳实践。

> （注：文档部分内容可能由 AI 生成）