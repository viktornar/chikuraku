"""Receive messages over from RabbitMQ and send them over the websocket."""

import pika
import os


credentials = pika.PlainCredentials(os.environ.get("RABBITMQ_DEFAULT_USER", "guest"), os.environ.get("RABBITMQ_DEFAULT_PASS", ""))

connection = pika.BlockingConnection(
    pika.ConnectionParameters(host=os.environ.get("RABBITMQ_HOST", "localhost"), port=5672, credentials=credentials)
)

channel = connection.channel()

channel.exchange_declare(
    exchange="ae41a5878bef411", exchange_type="fanout"
)

result = channel.queue_declare("", exclusive=True)
queue_name = result.method.queue
channel.queue_bind(exchange="ae41a5878bef411", queue=queue_name)

print("Listening for messages...")

while True:
    for method_frame, _, body in channel.consume(queue_name):
        try:
            print(body)
        except OSError as error:
            print(error)
        else:
            # acknowledge the message             
            channel.basic_ack(method_frame.delivery_tag)