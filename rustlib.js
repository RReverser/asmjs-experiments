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

  _embind_register_rust_char__deps: ['embind_repr', '$integerReadValueFromPointer', '$registerType'],
  _embind_register_rust_char: function(primitiveType) {
    registerType(primitiveType, {
        name: 'char',
        'fromWireType': function(value) {
          return String.fromCodePoint(value >>> 0);
        },
        'toWireType': function(destructors, value) {
          var valid = false;
          switch (typeof value) {
            case 'string':
              if (value.codePointAt(1) === undefined) {
                value = value.codePointAt(0);
                valid = true;
              }
              break;
            case 'number':
              if (Number.isInteger(value) && value >= 0 && value <= 0x10FFFF) {
                valid = true;
              }
              break;
          }
          if (!valid) {
            throw new TypeError('Cannot convert "' + _embind_repr(value) + '" to char');
          }
          return value;
        },
        'argPackAdvance': 8,
        'readValueFromPointer': function (pointer) {
          return String.fromCodePoint(HEAPU32[pointer >> 2]);
        },
        destructorFunction: null, // This type does not need a destructor
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

  _emval_array_push__deps: ['$requireHandle'],
  _emval_array_push: function (destHandle, elemHandle) {
    requireHandle(destHandle).push(requireHandle(elemHandle));
  }
});
