class Animal {
   constructor(public name) { }
   move(meters) {
       console.log(this.name + " moved " + meters + "m.");
   }
}

class Snake extends Animal {
   move() {
       console.log("Slithering...");
       super.move(5);
   }
}

class Horse extends Animal {
   move() {
       console.log("Galloping...");
       super.move(45);
   }
}

var sam = new Snake("Sammy the Python")
var tom: Animal = new Horse("Tommy the Palomino")

sam.move()
tom.move(34)

class Shape {
    private  static getLength(shape: number[]): number {
        let mul = 1;
        for (let dim of shape) {
            mul *= dim;
        }
        return mul;
    }

    private readonly _length: number;
    private readonly _rank: number;
    private readonly _shape: number[];

    get length() {
        return this._length;
    }

    get rank() {
        return this._rank;
    }

    get shape() {
        return this._shape;
    }


    constructor(shape: number[]) {
        this._shape = shape;
        this._rank = shape.length;
        this._length = Shape.getLength(shape);
    }
}