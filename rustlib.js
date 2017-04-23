mergeInto(LibraryManager.library, {
  _embind_register_rust_string__deps: ['$registerType'],
  _embind_register_rust_string: function(rawType) {
    registerType(rawType, {
        name: "&str",
        'argPackAdvance': 8,
        'readValueFromPointer': function (pointer) {
          pointer >>= 2;
          var length = HEAPU32[pointer + 1];
          pointer = HEAPU32[pointer];
          return Pointer_stringify(pointer, length);
        }
    });
  },

  _embind_iterator__deps: ['$requireHandle', '_emval_register'],
  _embind_iterator_start: function(handle) {
    handle = requireHandle(handle);
    return __emval_register(handle[Symbol.iterator]());
  },

  _embind_iterator_next__deps: ['$requireHandle', '_emval_register'],
  _embind_iterator_next: function(handle) {
    var next = requireHandle(handle).next();
    return next.done ? 0 : __emval_register(next.value);
  },

  _emval_get_string__deps: ['$requireHandle'],
  _emval_get_string: function(dest, handle) {
    handle = requireHandle(handle) + '';
    dest >>= 2;
    var length = HEAPU32[dest + 1] = lengthBytesUTF8(handle);
    var pointer = HEAPU32[dest] = _malloc(length + 1);
    stringToUTF8(handle, pointer, length + 1);
  },
});
