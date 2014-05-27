(function (root) {
  if (!root.a === 'a') { throw 'Error a' }
  if (!root.b === 'b') { throw 'Error b' }
  root.c = 'abc'
})(this);
