mergeInto(LibraryManager.library, {
  _embind_register_rust_string__deps: ['$registerType'],
  _embind_register_rust_string: function(rawType) {
    registerType(rawType, {
        name: "&str",
        'argPackAdvance': 8,
        'readValueFromPointer': function (pointer) {
            var length = HEAPU32[(pointer >> 2) + 1];
            pointer = HEAPU32[pointer >> 2];
            return Pointer_stringify(pointer, length);
        }
    });
  },

  _embind_iterator__deps: ['_emval_decref', '$requireHandle', '_emval_register'],
  _embind_iterator_start: function(handle) {
    handle = requireHandle(handle);
    return __emval_register(handle[Symbol.iterator]());
  },

  _embind_iterator_next__deps: ['$requireHandle', '_emval_register'],
  _embind_iterator_next: function(handle) {
    var next = requireHandle(handle).next();
    return next.done ? 0 : __emval_register(next.value);
  },
});
