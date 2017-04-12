mergeInto(LibraryManager.library, {
  _embind_register_rust_string__deps: ['$readLatin1String', '$registerType'],
  _embind_register_rust_string: function(rawType, name) {
    name = readLatin1String(name);
    registerType(rawType, {
        name: name,
        'argPackAdvance': 8,
        'readValueFromPointer': function (pointer) {
            var length = HEAPU32[(pointer >> 2) + 1];
            pointer = HEAPU32[pointer >> 2];
            return new TextDecoder().decode(HEAPU8.subarray(pointer, pointer + length));
        }
    });
  }
});
