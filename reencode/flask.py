from flask import Flask, request
app = Flask(__name__)


@app.route('/')
def hello_world():
    return 'Hello, World!'


@app.route('/reencode', methods=['POST'])
def reencode():
    if request.is_json:
        return "It's json! file={}".format(request.json['file'])
    else:
        return "It's not json. file={}".format(request.args)
