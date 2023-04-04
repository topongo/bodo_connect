```json
[
  {
    "subdomain": "earth.orbit",
    "eip": "1.2.3.4",
    "hosts": [
      {
        "name": "earth",
        "uuid": "---",
        "ip": "10.0.0.1",
        "port": 22,
        // `earth` is a network master, because it has an eport (external port)
        "eport": 22,
        "user": "human"
      },
      {
        "name": "moon",
        "ip": "10.0.0.2",
        "uuid": "---",
        "port": 444,
        "user": "rock",
        // set a GET waker for host `moon`
        "waker": {
          "method": "get",
          "url": "https://earth.orbit/api/v1/wake/moon/"
        }
      }
    ]
  },
  {
    "subdomain": "mars.orbit",
    "eip": null,
    "hosts": [
      {
        "name": "mars",
        "uuid": "---",
        "ip": "192.168.1.1",
        "port": 22,
        "eport": 22,
        "user": "martian"
      },
      {
        "name": "phobos",
        "ip": "192.168.1.2",
        "uuid": "---",
        "port": 444,
        "user": "rover",
        // set a wol waker for host `moon`
        "waker": {
          "mac": "00:08:55:05:ef"
        }
      }
    ]
  }
]
```