// Simple debug script

function test() {
    console.log(this.name);
}

var a = {
    name: 'kaka'
};

test.apply(a);