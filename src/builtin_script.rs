pub const BUILDIN_SCRIPT: &str = r#"
    Object.defineProperty(Array.prototype, "length", {
        "writable": false,
        "configurable": false,
        "get": function() {
            return this.__len__(); },
        });
"#;
