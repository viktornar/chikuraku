import StompJs from "stompjs";

class Stomp {
  constructor(username = "admin", password = "adminpass", debug = false) {
    this.client = StompJs.client("ws://localhost:15674/ws");
    this.client.connect(
      username,
      password,
      () => {},
      () => {}
    );
    if (debug) {
      this.client.debug = (str) => {
        console.debug(`# ${str}\n`);
      };
    }
  }

  onMessagesListener = (chatSessionId, callback) => {
    let subscription = { unsubscribe: () => {} };

    if (chatSessionId) {
      subscription = this.client.subscribe(
        `/exchange/amq.topic/${chatSessionId}`,
        callback
      );
    }

    return subscription;
  };
}

export default Stomp;
