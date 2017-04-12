mergeInto(LibraryManager.library, {
  _embind_register_rust_string__deps: ['$registerType'],
  _embind_register_rust_string: function(rawType) {
    registerType(rawType, {
        name: "&'static str",
        'argPackAdvance': 8,
        'readValueFromPointer': function (pointer) {
            var length = HEAPU32[(pointer >> 2) + 1];
            pointer = HEAPU32[pointer >> 2];
            return Pointer_stringify(pointer, length);
        }
    });
  }
});
