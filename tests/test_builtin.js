
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

test_proto();