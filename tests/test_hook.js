
var a = new_hook("kaka");
var b = a;

assert( show_hooks() == 1, " show_hooks 1");

a = null;

assert( show_hooks() == 1, " show_hooks 2");

b = null;

assert( show_hooks() == 0, " show_hooks 3");
