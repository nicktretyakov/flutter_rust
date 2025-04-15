import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:path_provider/path_provider.dart';

// FFI type definitions
typedef ProcessAudioNative = Void Function(
    Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>, Pointer, Pointer<Void>);
typedef ProcessAudio = void Function(
    Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>, Pointer, Pointer<Void>);

typedef RecognizeImageNative = Void Function(
    Pointer<Utf8>, Pointer<Utf8>, Pointer, Pointer<Void>);
typedef RecognizeImage = void Function(
    Pointer<Utf8>, Pointer<Utf8>, Pointer, Pointer<Void>);

typedef CallbackNative = Void Function(Pointer<Utf8>, Pointer<Void>);
typedef Callback = void Function(Pointer<Utf8>, Pointer<Void>);

// Rust backend interface
class RustBackend {
  late DynamicLibrary _lib;
  late ProcessAudio _processAudio;
  late RecognizeImage _recognizeImage;

  // Singleton pattern
  static final RustBackend _instance = RustBackend._internal();
  factory RustBackend() => _instance;

  RustBackend._internal() {
    _initLibrary();
  }

  void _initLibrary() {
    final libraryPath = _getLibraryPath();
    _lib = DynamicLibrary.open(libraryPath);

    _processAudio = _lib
        .lookupFunction<ProcessAudioNative, ProcessAudio>('process_audio');
    _recognizeImage = _lib
        .lookupFunction<RecognizeImageNative, RecognizeImage>('recognize_image');
  }

  String _getLibraryPath() {
    if (Platform.isAndroid) {
      return 'librust_backend.so';
    } else if (Platform.isIOS) {
      return 'rust_backend.framework/rust_backend';
    } else if (Platform.isWindows) {
      return 'rust_backend.dll';
    } else if (Platform.isLinux) {
      return 'librust_backend.so';
    } else if (Platform.isMacOS) {
      return 'librust_backend.dylib';
    }
    throw UnsupportedError('Unsupported platform');
  }

  // Process audio file and get GPT response
  Future<String> processAudioFile(String apiKey, String audioPath) async {
    final completer = Completer<String>();

    // Get a temporary file path for the response
    final tempDir = await getTemporaryDirectory();
    final outputPath = '${tempDir.path}/response_${DateTime.now().millisecondsSinceEpoch}.txt';

    // Prepare FFI arguments
    final apiKeyPtr = apiKey.toNativeUtf8();
    final audioPathPtr = audioPath.toNativeUtf8();
    final outputPathPtr = outputPath.toNativeUtf8();

    // Callback to handle the response
    void callback(Pointer<Utf8> resultPtr, Pointer<Void> userData) {
      final result = resultPtr.toDartString();
      malloc.free(resultPtr);

      if (result.startsWith('Error:')) {
        completer.completeError(Exception(result));
      } else {
        // Read the response file
        try {
          final responseText = File(outputPath).readAsStringSync();
          completer.complete(responseText);
        } catch (e) {
          completer.completeError(e);
        }
      }
    }

    // Register the callback
    final callbackPtr = Pointer.fromFunction<CallbackNative>(callback);

    // Call the Rust function
    _processAudio(apiKeyPtr, audioPathPtr, outputPathPtr, callbackPtr, nullptr);

    // Clean up
    malloc.free(apiKeyPtr);
    malloc.free(audioPathPtr);
    malloc.free(outputPathPtr);

    return completer.future;
  }

  // Recognize objects in image
  Future<String> recognizeImage(String apiKey, String imagePath) async {
    final completer = Completer<String>();

    // Prepare FFI arguments
    final apiKeyPtr = apiKey.toNativeUtf8();
    final imagePathPtr = imagePath.toNativeUtf8();

    // Callback to handle the response
    void callback(Pointer<Utf8> resultPtr, Pointer<Void> userData) {
      final result = resultPtr.toDartString();
      malloc.free(resultPtr);
      completer.complete(result);
    }

    // Register the callback
    final callbackPtr = Pointer.fromFunction<CallbackNative>(callback);

    // Call the Rust function
    _recognizeImage(apiKeyPtr, imagePathPtr, callbackPtr, nullptr);

    // Clean up
    malloc.free(apiKeyPtr);
    malloc.free(imagePathPtr);

    return completer.future;
  }
}
