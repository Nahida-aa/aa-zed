---
name: rust-cargo
description: cargo 使用指南
---

## check

```sh
# 只看错误
CARGO_BUILD_WARNINGS=allow cargo check -p aacode
```

看谁直接依赖 language_models

```sh
cargo tree --workspace -i language_models --depth 1
```
