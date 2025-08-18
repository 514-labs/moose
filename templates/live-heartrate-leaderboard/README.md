# Get started

1. Install Moose and sloan:\
   `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose, sloan`\
   Ensure you have followed the instructions adding moose to your path or start a new terminal

2. Create a python virtual environment
   ```
   python3 -m venv .venv
   source .venv/bin/activate
   ```

3. Install dependencies\
   `pip install -r requirements.txt`

4. Run Moose\
   `moose dev`

5. Start workflows\
   In another terminal run:\
   moose workflow run generate_data`

6. Configure Sloan:\
   `sloan setup --mcp cursor-project`
   `sloan setup --mcp claude-desktop`

7. Running Streamlit:\
   In another terminal:\
   `streamlit run streamlit_app.py`

## SOS

`docker stop $(docker ps -aq)`

# Deploy on Boreal

The easiest way to deploy your Moose app is to use [Boreal](https://www.fiveonefour.com/boreal) from 514 Labs, the creators of Moose.

Check out our [Moose deployment documentation](https://docs.fiveonefour.com/moose/deploying) for more details.
