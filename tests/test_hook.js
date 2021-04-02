
function test_hook1() {
    var a = new_hook("kaka");
    var b = a;
    assert( show_hooks() == 1, " show_hooks 1");

    a = null;
    assert( show_hooks() == 1, " show_hooks 2");
    
    print_hook(b);
    b = null;
    assert( show_hooks() == 0, " show_hooks 3");

    println("-------- END TESTING -----------");
}

test_hook1();