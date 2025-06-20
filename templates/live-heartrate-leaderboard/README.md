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

5. Generate mock ANT+ Data
In another terminal run:
```moose workflow run generate_data```


6. Configure Aurora:
```aurora setup --mcp cursor-project```
```aurora setup --mcp claude-desktop```

7. Running Streamlit:
In another terminal:
```streamlit run app/streamlit_app.py```

## Additional - Connect your own Heart Rate Device (Apple Watch)

Connect your own Apple Watch to the system. This has only been tested on the AppleWatch SE! Please feel free to add your device, following the example in `app/scripts/apple_watch_hr_client` 

1. Download the Echo App on your IPhone/Android device since the Apple Watch doesn't nativley support broadcasting HR data over BLE
2. Update the name in the code to match your device name (or a substring)  `app/scripts/apple_watch_hr_client/apple_heart_rate_client` 
3. Update the "user table" in the `mock-user-db.json` to match your details. 
4. Run your `moose dev` server and then run `moose workflow run apple_watch_hr_client`


To terminate the (temporal) workflow, run:

`moose workflow terminate apple_watch_hr_client`
