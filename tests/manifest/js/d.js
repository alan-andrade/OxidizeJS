(function (root) {
  if ('undefined' === typeof a) {
    throw 'This requires that a is defined'
  } else {
    console.log('It works! Value of a:' a);
  }
})(this)
