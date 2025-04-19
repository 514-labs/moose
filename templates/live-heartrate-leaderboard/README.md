# Get started
1. Install Moose and aurora:
```bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose, aurora```
Ensure you have followed the instructions adding moose to your path or start a new terminal

2. Create a python virtual environment
```python3 -m venv .venv
source .venv/bin/activate```

3 Install dependencies
```pip install -r requirements.txt```

4. Run Moose
```moose dev```

5. Start workflows
In another terminal run:
```moose workflow run generate_data```


6. Configure Aurora:
```aurora setup --mcp cursor-project```
```aurora setup --mcp claude-desktop```

7. Running Streamlit:
In another terminal:
```streamlit run app/streamlit_app.py```

## SOS 

```docker stop $(docker ps -aq)```
