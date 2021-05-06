// Simple debug script

function check_dims( dims ) {
    var n = dims.length;
    var check = [];
    for (var i = 0; i < n; i++) {
        if (dims[i] < 0 || dims[i] >= n) {
            throw new Error("Error: out of dimensition!");
        }
        
        for (var j = 0; j < check.length; j++) {
            if (check[j] == dims[i]) {
                throw new Error("Error: repeated dimensition!");
            }
        }
        check.push( dims[i] );
    }
    return true;
}

check_dims( [1,2,5,0] );
