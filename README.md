# vmm_memory

__Build the tool__
```
cargo build
```

__Get overhead based on guest memory `name`__
```
sudo ./target/debug/vmm_memory [vmm PID] --name=[guest memory name]
```

__Get overhead based on guest memory `size`__
```
sudo ./target/debug/vmm_memory [vmm PID] --size=[guest memory size in kiB]
```
