function test_while()
{
    var i, c;
    i = 0;
    c = 0;
    while (i < 3) {
        c++;
        i++;
    }
    assert(c === 3, "while 1");

    console.log("-------- END TESTING -----------");
}

function test_while_break()
{
    var i, c;
    i = 0;
    c = 0;
    while (i < 3) {
        c++;
        if (i == 1)
            break;
        i++;
    }
    assert(c === 2 && i === 1, "while break 1");

    console.log("-------- END TESTING -----------");
}

function test_do_while()
{
    var i, c;
    i = 0;
    c = 0;
    do {
        c++;
        i++;
    } while (i < 3);
    assert(c === 3 && i === 3, "do while 1");

    console.log("-------- END TESTING -----------");
}

function test_for()
{
    var i, c;
    c = 0;
    for(i = 0; i < 3; i++) {
        c++;
    }
    assert(c === 3 && i === 3, "for 1");

    c = 0;
    for(var j = 0; j < 3; j++) {
        c++;
    }
    assert(c === 3 && j === 3, "for 2");

    console.log("-------- END TESTING -----------");
}

function test_for_in()
{
    var i, tab, a, b;

    tab = [];
    for(i in {x:1, y: 2}) {
        tab.push(i);
    }
    assert( (tab[1] == "y" && tab[0] == "x") || (tab[0] == "y" && tab[1] == "x"), "for in 1");

    /* prototype chain test */
    a = {x:1, y: 2, "1": 3};
    b = {"4" : 4 };
    Object.setPrototypeOf(a, b);
    assert(a["4"] == 4, "prototype 1");

    /* non enumerable properties hide enumerables ones in the
       prototype chain */
    a = {y: 2, "1": 3};    
    b = {"x" : 3 };
    Object.setPrototypeOf(a, b);
    tab = [];
    for(i in a) {
        tab.push(i);
    }
    a = tab.toString();
    assert(a == "1, y" || a == "y, 1", "for_in 2");
    
    // TODO
    /* array optimization */
    /* 
    a = [];
    for(i = 0; i < 10; i++)
        a.push(i);
    tab = [];
    for(i in a) {
        tab.push(i);
    }
    assert(tab.toString(), "0,1,2,3,4,5,6,7,8,9", "for_in");
    */


    /* variable definition in the for in */
    tab = [];
    for(var j in {x:1, y: 2}) {
        tab.push(j);
    }
    assert(tab.toString() == "x, y" || tab.toString() == "y, x", "for_in 3");

    console.log("-------- END TESTING -----------");
}

function test_for_in2()
{
    var i;
    tab = [];
    for(i in {x:1, y: 2, z:3}) {
        if (i === "y")
            continue;
        tab.push(i);
    }
    assert(tab.toString() == "x, z" || tab.toString() == "z, x", "for in 2");

    tab = [];
    for(i in {x:1, y: 2, z:3}) {
        if (i === "z")
            break;
        tab.push(i);
    }
    assert(tab.toString() == "x, y" || tab.toString() == "y, x" || tab.toString() == "y" || tab.toString() == "x" || tab.toString() == "" , "for in 2");

    console.log("-------- END TESTING -----------");
}

function test_for_break()
{
    var i, c;
    c = 0;
    L1: for(i = 0; i < 3; i++) {
        c++;
        if (i == 0)
            continue;
        while (1) {
            break L1;
        }
    }
    assert(c === 2 && i === 1, "for break");

    console.log("-------- END TESTING -----------");
}

function test_switch1()
{
    var i, a, s;
    s = "";
    for(i = 0; i < 3; i++) {
        a = "?";
        switch(i) {
        case 0:
            a = "a";
            break;
        case 1:
            a = "b";
            break;
        default:
            a = "c";
            break;
        }
        s += a;
    }
    assert(s === "abc" && i === 3, "switch 1");

    console.log("-------- END TESTING -----------");
}

function test_switch2()
{
    var i, a, s;
    s = "";
    for(i = 0; i < 4; i++) {
        a = "?";
        switch(i) {
        case 0:
            a = "a";
            break;
        case 1:
            a = "b";
            break;
        case 2:
            continue;
        default:
            a = "" + i;
            break;
        }
        s += a;
    }
    assert(s === "ab3" && i === 4, "switch 2");

    console.log("-------- END TESTING -----------");
}

function test_try_catch1()
{
    try {
        throw Error("Hello");
    } catch (e) {
        assert(e.message() == "Hello", "catch 1");
        return;
    }
    assert(false, "catch 2");

    console.log("-------- END TESTING -----------");
}

function test_try_catch2()
{
    var a;
    try {
        a = 1;
    } catch (e) {
        a = 2;
    }
    assert(a ==  1, "catch 3");

    console.log("-------- END TESTING -----------");
}

function test_try_catch3()
{
    var s;
    s = "";
    try {
        s += "t";
    } catch (e) {
        s += "c";
    } finally {
        s += "f";
    }
    assert(s == "tf", "catch 4");

    console.log("-------- END TESTING -----------");
}

function test_try_catch4()
{
    var s;
    s = "";
    try {
        s += "t";
        throw Error("c");
    } catch (e) {
        s += e.message();
    } finally {
        s += "f";
    }
    assert(s == "tcf", "catch 5");

    console.log("-------- END TESTING -----------");
}

function test_try_catch5()
{
    var s;
    s = "";
    for(;;) {
        try {
            s += "t";
            break;
            s += "b";
        } finally {
            s += "f";
        }
    }
    assert(s == "tf", "catch 6");

    console.log("-------- END TESTING -----------");
}

function test_try_catch6()
{
    function f() {
        try {
            s += 't';
            return 1;
        } finally {
            s += "f";
        }
    }
    var s = "";
    assert(f() == 1, "catch 7");
    assert(s == "tf", "catch 8");

    console.log("-------- END TESTING -----------");
}

function test_try_catch7()
{
    var s;
    s = "";

    try {
        try {
            s += "t";
            throw Error("a");
        } finally {
            s += "f";
        }
    } catch(e) {
        s += e.message();
    } finally {
        s += "g";
    }
    assert(s == "tfag", "catch 9");
    console.log("-------- END TESTING -----------");
}

function test_try_catch8()
{
    var i, s;
    
    s = "";
    for(var i in {x:1, y:2}) {
        try {
            s += i;
            throw Error("a");
        } catch (e) {
            s += e.message();
        } finally {
            s += "f";
        }
    }
    assert(s == "xafyaf" || s == "yafxaf", "catch 10");    
    console.log("-------- END TESTING -----------");
}

test_while();
test_while_break();
test_do_while();
test_for();
test_for_break();
test_switch1();
test_switch2();
test_for_in();
test_for_in2();

test_try_catch1();
test_try_catch2();
test_try_catch3();
test_try_catch4();
test_try_catch5();
test_try_catch6();
test_try_catch7();
test_try_catch8();
