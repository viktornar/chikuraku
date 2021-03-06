"""Notification channels for django-notifs."""

from json import dumps

import pika
import os

from notifications.channels import BaseNotificationChannel


class BroadCastWebSocketChannel(BaseNotificationChannel):
    """Fanout notification for RabbitMQ."""

    def _connect(self):
        """Connect to the RabbitMQ server."""

        credentials = pika.PlainCredentials(os.environ.get("RABBITMQ_DEFAULT_USER", "guest"), os.environ.get("RABBITMQ_DEFAULT_PASS", ""))

        connection = pika.BlockingConnection(
            pika.ConnectionParameters(host=os.environ.get("RABBITMQ_HOST", "localhost"), port=5672, credentials=credentials)
        )
        channel = connection.channel()

        return connection, channel

    def construct_message(self):
        """Construct the message to be sent."""
        extra_data = self.notification_kwargs['extra_data']

        return dumps(extra_data['message'])

    def notify(self, message):
        """put the message of the RabbitMQ queue."""
        connection, channel = self._connect()

        uri = self.notification_kwargs['extra_data']['uri']

        channel.exchange_declare(exchange='ampq.fanout', exchange_type='fanout')
        channel.basic_publish(exchange='ampq.fanout', routing_key=uri, body=message)

        connection.close()