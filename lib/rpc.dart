import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:http/http.dart' as http;

import 'package:esse/utils/websocket/MyWsChannel.dart';
import 'package:esse/global.dart';

Map jsonrpc = {
  "jsonrpc": "2.0",
  "id": 1,
  "gid": Global.gid,
  "method": "",
  "params": [],
};

class Response {
  final bool isOk;
  final List params;
  final String error;

  const Response({this.isOk, this.params, this.error});
}

Future<Response> httpPost(String addr, String method, List params) async {
  jsonrpc['method'] = method;
  jsonrpc['params'] = params;
  //print(json.encode(jsonrpc));

  try {
    final response = await http.post(Uri.http(addr, ''), body: json.encode(jsonrpc));
    Map data = json.decode(utf8.decode(response.bodyBytes));

    if (data['result'] != null) {
      return Response(isOk: true, params: data['result'], error: '');
    } else {
      return Response(isOk: false, params: [], error: data['error']['message']);
    }
  } catch (e) {
    print(e);
    return Response(isOk: false, params: [], error: 'network error');
  }
}

WebSocketsNotifications rpc = new WebSocketsNotifications();

class WebSocketsNotifications {
  static final WebSocketsNotifications _sockets =
      new WebSocketsNotifications._internal();

  factory WebSocketsNotifications() {
    return _sockets;
  }

  WebSocketsNotifications._internal();

  WebSocketChannel _channel;

  bool _closed = true;

  Map<String, List> _listeners = new Map<String, List>();
  Function _notice;

  bool isLinked() {
    return !_closed;
  }

  Future<bool> init(String addr) async {
    reset();

    var i = 0;

    while (true) {
      try {
        _channel = await MyWsChannel.connect(Uri.parse('ws://' + addr));
        _closed = false;
        _channel.stream.listen(
          _onReceptionOfMessageFromServer,
          cancelOnError: true,
          onDone: () {
            String closeReason = "";
            try {
              closeReason = _channel.closeReason.toString();
            } catch (_) {}
            print("WebSocket done… " + closeReason);
            _closed = true;
        });
        return true;
      } catch (e) {
        print("DEBUG Flutter: got websockt error.........retry");
        //print(e);
        if (i > 3) {
          return false;
        }

        i += 1;
        await Future.delayed(Duration(seconds: 1), () => true);
        continue;
      }
    }
  }

  reset() {
    if (_channel != null) {
      if (_channel.sink != null) {
        _channel.sink.close();
      }
    }
    _closed = true;
  }

  send(String method, List params) {
    jsonrpc["method"] = method;
    jsonrpc["params"] = params;
    jsonrpc["gid"] = Global.gid;

    if (_channel != null) {
      if (_channel.sink != null) {
        _channel.sink.add(json.encode(jsonrpc));
      }
    }
  }

  addNotice(Function callback) {
    _notice = callback;
  }

  addListener(String method, Function callback, bool notice) {
    _listeners[method] = [callback, notice];
  }

  removeListener(String method) {
    _listeners.remove(method);
  }

  _onReceptionOfMessageFromServer(message) {
    Map response = json.decode(message);
    print(response);

    if (response["result"] != null &&
        response["result"].length != 0 &&
        response["method"] != null &&
        response["gid"] != null
      ) {
        String method = response["method"];
        List params = response["result"];
        String gid = response["gid"];
      if (_listeners[method] != null) {
        if (gid == Global.gid || method.startsWith('account')) {
          _listeners[method][0](params);
        } else {
          _notice(gid);
        }
      } else {
        print("has no this " + method);
      }
    }
  }
}
