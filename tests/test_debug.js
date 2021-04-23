// Simple debug script

var OneClass = function() {
    this.x = "haha";
}
OneClass.prototype.test = function(y) {
    console.log(y);
    console.log(this.x);
    return this.x;
};

var a = new OneClass();
a.test.call(a, 3.14);