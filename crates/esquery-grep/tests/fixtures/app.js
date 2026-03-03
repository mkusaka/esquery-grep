function greet(name) {
  const message = "Hello, " + name;
  console.log(message);
  return message;
}

var x = 42;
if (x > 10) {
  greet("world");
}
