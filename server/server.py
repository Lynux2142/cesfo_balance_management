import csv
from jinja2 import Template
from os import getenv
from os.path import exists
from flask import Flask

FILE_PATH = "/data/balance_history.txt"

app = Flask(__name__)

@app.route("/")
def index():
    with open(FILE_PATH, newline="") as f:
        reader = csv.reader(f, delimiter=";")
        data = list(reader)

    template = Template("""
        <html>
            <head>
                <style>
                    table {
                        border-collapse: collapse;
                        margin-bottom: 50vh
                    }
                    td, th {
                        text-align: center;
                        padding: 10px;
                    }
                    thead {
                        position: sticky;
                        top: 0;
                        background-color: #4169e1;
                    }
                    tbody tr:nth-child(even) {
                        background-color: #b3d9ff;
                    }
                    tbody tr:nth-child(odd) {
                        background-color: #ccf2ff;
                    }
                </style>
            </head>
            <body onload="scrollToBottom()">
                <table id="table">
                    <thead>
                        <tr>
                            {% for col in data[0] %}
                                <th>{{ col }}</th>
                            {% endfor %}
                        </tr>
                    </thead>
                    <tbody>
                        {% for row in data[1:] %}
                            <tr>
                                {% for col in row %}
                                    <td>{{ col }}</td>
                                {% endfor %}
                            </tr>
                        {% endfor %}
                    </tbody>
                </table>
                <script>
                    function scrollToBottom() {
                        window.scrollTo(0, document.body.scrollHeight)
                    }
                </script>
            </body>
        </html>
    """)

    html = template.render(data=data)
    return html

if __name__ == "__main__":
    app.run()
