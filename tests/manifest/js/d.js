(function (root) {
  if ('undefined' === typeof root.a) {
    throw 'This requires that a is defined'
  } else {
    throw 'root.a is cool'
  }
})(this)
