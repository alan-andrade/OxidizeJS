(function (root) {
  if (root.a === 'a') {
    root.b = 'b';
  } else {
    throw 'Error a on b'
  }
})(this);
