
var log_str = "";

function log(str)
{
    log_str += str + ",";
}

function f(a, b, c)
{
    var x = 10;
    log("a="+a);
    function g(d) {
        function h() {
            log("d=" + d);
            log("x=" + x);
        }
        log("b=" + b);
        log("c=" + c);
        h();
    }
    g(4);
    return g;
}

var g1 = f(1, 2, 3);
g1(5);

assert(log_str == "a=1,b=2,c=3,d=4,x=10,b=2,c=3,d=5,x=10,", "closure 1");


var Greeting = /** @class */ (function () {
    function Greeting() {
        this.message = "KaKa";
    }
    Greeting.prototype.greet = function () {         
        this.message = "XaXa";
    };
    return Greeting;
}());
var obj = new Greeting();
assert(obj.message == "KaKa", " prototype 1");
obj.greet();
assert(obj.message == "XaXa", " prototype 2");


function test_closure1()
{
    function f2()
    {
        var val = 1;
        
        function set(a) {
            val = a;
        }
        function get(a) {
            return val;
        }
        return { "set": set, "get": get };
    }
    
    var obj = f2();
    obj.set(10);
    var r;
    r = obj.get();
    assert(r == 10, "closure 2");

    console.log("-------- END TESTING -----------");
}

function test_closure2()
{
    var expr_func = function myfunc1(n) {
        function myfunc2(n) {
            return myfunc1(n - 1);
        }
        if (n == 0) {
            return 0;
        } else {
            return myfunc2(n);
        }
    };
    var r;
    r = expr_func(1);
    assert(r == 0, "expr_func");

    console.log("-------- END TESTING -----------");
}

function test_closure3()
{
    function fib(n)
    {
        if (n <= 0) {
            return 0;
        } else if (n == 1) {
            return 1;
        } else
            return fib(n - 1) + fib(n - 2);
    }


    var fib_func = function fib1(n)
    {
        if (n <= 0) {
            return 0;
        } else if (n == 1) {
            return 1;
        } else
            return fib1(n - 1) + fib1(n - 2);
    };


    assert(fib(6) == 8, "fib");
    assert(fib_func(6) == 8, "fib_func");

    console.log("-------- END TESTING -----------");
}

test_closure1();
test_closure2();
test_closure3();
