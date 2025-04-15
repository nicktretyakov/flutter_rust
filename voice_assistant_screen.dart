import 'dart:io';
import 'package:flutter/material.dart';
import 'package:speech_to_text/speech_to_text.dart' as stt;
import 'package:flutter_tts/flutter_tts.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import './flutter_interface.dart';

class VoiceAssistantScreen extends StatefulWidget {
  @override
  _VoiceAssistantScreenState createState() => _VoiceAssistantScreenState();
}

class _VoiceAssistantScreenState extends State<VoiceAssistantScreen> {
  final stt.SpeechToText _speech = stt.SpeechToText();
  final FlutterTts _flutterTts = FlutterTts();
  final RustBackend _rustBackend = RustBackend();

  bool _isListening = false;
  String _text = "Press the button and start speaking";
  String _response = "";
  bool _isProcessing = false;

  @override
  void initState() {
    super.initState();
    _initSpeech();
    _initTts();
  }

  void _initSpeech() async {
    bool available = await _speech.initialize(
      onStatus: (status) {
        print('Speech status: $status');
        if (status == 'done') {
          setState(() {
            _isListening = false;
          });
          _processSpeech();
        }
      },
      onError: (errorNotification) {
        print('Speech error: $errorNotification');
        setState(() {
          _isListening = false;
        });
      },
    );

    if (!available) {
      setState(() {
        _text = "Speech recognition not available";
      });
    }
  }

  void _initTts() async {
    await _flutterTts.setLanguage("en-US");
    await _flutterTts.setSpeechRate(0.5);
    await _flutterTts.setVolume(1.0);
    await _flutterTts.setPitch(1.0);
  }

  void _listen() async {
    if (!_isListening) {
      setState(() {
        _text = "Listening...";
        _response = "";
        _isListening = true;
      });

      await _speech.listen(
        onResult: (result) {
          setState(() {
            _text = result.recognizedWords;
          });
        },
      );
    } else {
      setState(() {
        _isListening = false;
      });
      _speech.stop();
    }
  }

  void _processSpeech() async {
    if (_text.isEmpty || _text == "Listening...") return;

    setState(() {
      _isProcessing = true;
    });

    try {
      // Save speech to temporary file
      final tempDir = await getTemporaryDirectory();
      final audioFile = File(path.join(tempDir.path, 'speech.wav'));

      // This is just a placeholder - in a real app, you'd convert the speech to a WAV file
      // For demonstration, we'll use the text directly

      // Get API key from secure storage (in a real app)
      final apiKey = const String.fromEnvironment('OPENAI_API_KEY',
          defaultValue: 'YOUR_API_KEY_HERE');

      // Process with Rust backend
      final response = await _rustBackend.processAudioFile(
        apiKey,
        audioFile.path
      );

      setState(() {
        _response = response;
        _isProcessing = false;
      });

      // Speak the response
      await _flutterTts.speak(_response);

    } catch (e) {
      print('Error processing speech: $e');
      setState(() {
        _response = "Error: $e";
        _isProcessing = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Voice Assistant'),
      ),
      body: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          children: [
            Card(
              elevation: 4,
              child: Padding(
                padding: EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('You said:', style: TextStyle(fontWeight: FontWeight.bold)),
                    SizedBox(height: 8),
                    Text(_text, style: TextStyle(fontSize: 16)),
                  ],
                ),
              ),
            ),
            SizedBox(height: 16),
            if (_isProcessing)
              CircularProgressIndicator()
            else if (_response.isNotEmpty)
              Card(
                elevation: 4,
                color: Colors.blueGrey[50],
                child: Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Assistant:', style: TextStyle(fontWeight: FontWeight.bold)),
                      SizedBox(height: 8),
                      Text(_response, style: TextStyle(fontSize: 16)),
                    ],
                  ),
                ),
              ),
            Spacer(),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _isProcessing ? null : _listen,
        tooltip: 'Listen',
        child: Icon(_isListening ? Icons.mic : Icons.mic_none),
        backgroundColor: _isListening ? Colors.red : null,
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.centerFloat,
    );
  }

  @override
  void dispose() {
    _speech.stop();
    _flutterTts.stop();
    super.dispose();
  }
}
