# Verilator Runtime

- All models are "owned" by the runtime. Lifetimes enforce that they cannot outlive it. All deallocation of Verilated models is done when the runtime is dropped --- when an individual model is dropped, no deallocation occurs. This allows, for instance, for constructing a model with a struct initializer, where `Alu { ..alu }` would otherwise have deallocated the model when dropping the old `alu` and caused a double-free error when the newly-constructed model was also dropped.
