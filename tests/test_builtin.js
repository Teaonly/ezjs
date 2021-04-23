
function test_proto()
{
    assert(String.prototype.proto() === Object.prototype, "1111");

    var a = {name: 'kaka'};
    assert(a.proto() === Object.prototype, "2222");
    assert(a.constructor === Object);

    var a = "haha";
    assert(a.proto() === String.prototype, "33330");
    assert(a.constructor === String, "33331");
    assert(a.proto().constructor === String, "33332");
    assert(a.proto() !== Object.prototype, "33333");

    function OneClass() {
        this.x = 3.14;
    }

    var a = new OneClass();
    var b = new OneClass();
    a.x = 6.28;
    assert( a.proto() === OneClass.prototype, "4444");
    assert( a.proto() === b.proto(), "5555");
    assert( a.constructor === OneClass , "6666");

    Object.setPrototypeOf(a, Object.prototype);
    assert( a.proto() === Object.prototype, "7777");
    assert( OneClass.prototype.proto() === Object.prototype, "8888");

    console.log("-------- END TESTING -----------");
}

// ==================================
function test_prototype() {
    var __extends = (this && this.__extends) || (function () {
        var extendStatics = function (d, b) {
            return Object.setPrototypeOf(d, b);
        };
        return function (d, b) {
            extendStatics(d, b);
            function __() {
                this.constructor = d;
            }
            d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
        };
    })();

    var Shape = /** @class */ (function () {
        function Shape(a) {
            this.Area = a;
        }
        return Shape;
    }());

    var Circle = (function (_super) {
        function Circle(a) {
            this.Area = a;
        }

        __extends(Circle, _super);

        Circle.prototype.disp = function () {
            console.log("Area of the circle:  " + this.Area);
        };

        return Circle;
    }(Shape));

    var c = new Circle(3.14);
    assert(c.disp() === undefined, "class 11111");
    assert(c.proto() === Circle.prototype, "class 22222");

    assert(Circle.proto() === Shape, "class 33333");

    assert(c.constructor === Circle, "class 44444");

    assert(Circle.prototype.constructor === Circle, "class 55555");
    assert(Circle.prototype.proto() === Shape.prototype, "class 66666");


    console.log("-------- END TESTING -----------");
}

function test_apply() {
    var OneClass = function() {
        this.x = "haha";
    }
    OneClass.prototype.test = function() {
        console.log(this.x);
        return this.x;
    };

    var a = new OneClass();
    var b = new OneClass();
    b.x = "fafa";
    assert(a.test.apply(a, []) == "haha",  "apply 1111");
    assert(a.test.apply(b, []) == "fafa",  "apply 2222");

    console.log("-------- END TESTING -----------");

}

test_proto();
test_prototype();
test_apply();
