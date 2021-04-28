.DEFAULT_GOAL := test_typescript.js
SOURCES := test_typescript.ts

test_typescript.js: $(SOURCES)
	tsc --target es5 test_typescript.ts

clean:
	rm -f test_typescript.js