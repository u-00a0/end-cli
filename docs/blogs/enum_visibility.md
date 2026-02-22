# rust语言设计的不严谨之处

虽然 Rust 一向以严谨著称，但是用了一段时间后，还是发现语言设计上有不少不严谨的地方，仅举几例。
    
## enum 分支的可见性无法单独标记 (2026/02/14)

### 类型不变量

类型不变量是一个谓词 Inv_T(x) : T → Prop，对所有良构且通过安全接口可达的 x:T 都成立，并由该类型的公开操作保持。比如下面这个例子中，Catalog 保持了 items 和 item_index 之间的同步，这就是一个类型不变量。

```rust
mod a {
    pub struct Catalog {
        /*没有 pub*/ items: Vec<Item>,
        /*没有 pub*/ item_index: HashMap<String, ItemId>,
    }

    impl Catalog {
        pub fn new() -> Self {
            // ...
        }

        // 维护 items 和 item_index 之间的同步
        pub fn add_item(&mut self, item: Item) {
            self.item_index.insert(item.name.clone(), item.id);
            self.items.push(item);
        }
    }
}
```

我们通过可见性限制来保证通过安全接口可达的 x:T 都满足类型不变量。

类似地，对于 enum，我们有 enum 不变量 Inv_E(x) : E → Prop，可以重写为类似下面的标准型：

```rust
fn inv_e(x: E) -> bool {
    match x {
        E::A(i, j) => /* 满足某些条件 */,
        E::B => /* 满足某些条件 */,
    }
}
```

此时我们重新审视 Rust 的和类型，发现它的分支没有可见性修饰符！

```rust
mod a {
    pub enum E {
        A(i32, i32), // 无法修饰为 private
        B,
    }
}

mod b {
    fn example(){
        let e = a::E::A(1, 2); // 可以直接构造
    }
}
```

这意味着 enum 无法拥有任何类型不变量！一个 workaround 是将 enum 的变体都newtype 包装起来。

