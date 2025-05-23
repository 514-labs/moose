import streamlit as st
import requests
import time
import plotly.graph_objects as go
import pandas as pd
from datetime import datetime, timezone
import numpy as np
import json

# Page config
st.set_page_config(
    page_title="Live Heart Rate Monitor",
    page_icon="❤️",
    layout="wide"
)

# Modern animated banner
# st.markdown("""
#     <style>
#         .modern-banner {
#             background: linear-gradient(135deg, #7f00ff, #e100ff);
#             color: white;
#             padding: 2rem;
#             text-align: center;
#             font-size: 1.7rem;
#             font-weight: 600;
#             font-family: 'Segoe UI', sans-serif;
#             border-radius: 15px;
#             box-shadow: 0 6px 18px rgba(0, 0, 0, 0.25);
#             animation: slideFadeIn 0.8s ease-out, pulse 2.5s ease-in-out infinite;
#             margin-bottom: 2rem;
#         }

#         @keyframes slideFadeIn {
#             0% {
#                 transform: translateY(-20px);
#                 opacity: 0;
#             }
#             100% {
#                 transform: translateY(0);
#                 opacity: 1;
#             }
#         }

#         @keyframes pulse {
#             0% {
#                 box-shadow: 0 0 12px rgba(255, 255, 255, 0.2);
#             }
#             50% {
#                 box-shadow: 0 0 28px rgba(255, 255, 255, 0.45);
#             }
#             100% {
#                 box-shadow: 0 0 12px rgba(255, 255, 255, 0.2);
#             }
#         }
#     </style>

#     <div class="modern-banner">
#         Go from prototype to production<br>
#         \n Made with ❤️ MOOSE X SINGLESTONE \n 
#             Learn More: docs.fiveonefour.com/moose
#         Download: bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose)
#     </div>
# """, unsafe_allow_html=True)

# Initialize session state and constants
if 'selected_user' not in st.session_state:
    st.session_state.selected_user = None
if 'hr_data' not in st.session_state:
    # Initialize with explicit dtypes to ensure consistency
    st.session_state.hr_data = pd.DataFrame({
        'timestamp': pd.Series(dtype='datetime64[ns, UTC]'),
        'heart_rate': pd.Series(dtype='float64'),
        'hr_zone': pd.Series(dtype='int64'),
        'estimated_power': pd.Series(dtype='float64'),
        'cumulative_calories_burned': pd.Series(dtype='float64')
    })

# Constants
LEADERBOARD_TIME_WINDOW = 300  # 5 minutes in seconds

# Ensure a default selected user (top of leaderboard) when the app loads
def ensure_default_selected_user():
    if st.session_state.selected_user is None:
        try:
            resp = requests.get(
                f"http://localhost:4000/consumption/getLeaderboard?time_window_seconds={LEADERBOARD_TIME_WINDOW}&limit=1"
            )
            if resp.status_code == 200 and resp.json().get("entries"):
                st.session_state.selected_user = resp.json()["entries"][0]["user_name"]
        except Exception:
            pass

ensure_default_selected_user()

# Helper: fetch profile image for a given user name from mock-user-db.json
def get_profile_image(user_name: str):
    """Return the profile image URL for the provided user name or None if not found."""
    try:
        with open("mock-user-db.json", "r") as f:
            user_db = json.load(f)
        for user in user_db.values():
            if user.get("user_name") == user_name:
                return user.get("profile_image")
    except Exception as e:
        # Use Streamlit's error display only if we are in a Streamlit context
        try:
            st.error(f"Failed to load user image: {str(e)}")
        except Exception:
            pass
    return None

# Function to update the live graph
def update_live_graph():
    try:
        if st.session_state.selected_user:
            response = requests.get(
                f"http://localhost:4000/consumption/getUserLiveHeartRateStats?user_name={st.session_state.selected_user}&window_seconds=60"
            )
            if response.status_code == 200:
                data = response.json()
                if data:
                    # Convert data to DataFrame
                    new_data = pd.DataFrame([{
                        'timestamp': datetime.fromisoformat(d['processed_timestamp'].replace('Z', '+00:00')),
                        'heart_rate': d['heart_rate'],
                        'hr_zone': d['hr_zone'],
                        'estimated_power': d['estimated_power'],
                        'cumulative_calories_burned': d['cumulative_calories_burned']
                    } for d in data])
                    
                    # Update session state data
                    if st.session_state.hr_data.empty:
                        st.session_state.hr_data = new_data
                    else:
                        st.session_state.hr_data = pd.concat([st.session_state.hr_data, new_data], ignore_index=True)
                    
                    # Drop duplicates and sort by timestamp
                    st.session_state.hr_data = st.session_state.hr_data.drop_duplicates(subset=['timestamp'])
                    st.session_state.hr_data = st.session_state.hr_data.sort_values('timestamp')
                    
                    # Keep only last 60 seconds of data
                    cutoff_time = datetime.now(timezone.utc) - pd.Timedelta(seconds=60)
                    st.session_state.hr_data = st.session_state.hr_data[st.session_state.hr_data['timestamp'] > cutoff_time]
                    
                    # Return the most recent data point
                    return st.session_state.hr_data.iloc[-1] if not st.session_state.hr_data.empty else None
    except Exception as e:
        st.error(f"Failed to update graph: {str(e)}")
    return None

# Function to update the leaderboard
def update_leaderboard():
    """Return HTML table (with avatars) + row count."""
    try:
        response = requests.get(
            f"http://localhost:4000/consumption/getLeaderboard?time_window_seconds={LEADERBOARD_TIME_WINDOW}&limit=100"
        )
        if response.status_code == 200:
            data = response.json()["entries"]
            df = pd.DataFrame(data)

            # Relevant columns for display (include max stats)
            display_cols = [
                "rank",
                "user_name",
                "avg_heart_rate",
                "max_heart_rate",
                "avg_power",
                "max_power",
                "total_calories",
            ]
            df_display = df[display_cols].copy()

            # Round numeric columns to 1 decimal place (excluding rank)
            numeric_cols = [
                "avg_heart_rate",
                "max_heart_rate",
                "avg_power",
                "max_power",
                "total_calories",
            ]
            df_display[numeric_cols] = df_display[numeric_cols].round(1)

            # Inject avatar HTML in a new first column and re-order
            df_display.insert(
                0,
                "avatar",
                df_display["user_name"].apply(
                    lambda u: f"<img src='{get_profile_image(u)}' style='width:32px;border-radius:50%;'>"
                ),
            )

            # Row-highlighting for the current selected user
            def highlight_selected_user(row):
                if row["user_name"] == st.session_state.selected_user:
                    return ["background-color: #FF4B4B30"] * len(row)
                return [""] * len(row)

            fmt_dict = {col: "{:.1f}" for col in numeric_cols}

            styled = (
                df_display.style.hide(axis="index")
                .format(fmt_dict)  # type: ignore[arg-type]
                .apply(highlight_selected_user, axis=1)
            )

            # Convert to raw HTML so avatar <img> tags render and widen table
            html_table = styled.to_html(escape=False).replace(
                "<table ",
                "<table style='width:100%; table-layout:auto;' ",
            )

            # Update selected user to the new leader (rank 1) if different
            leader = df_display.iloc[0]["user_name"] if not df_display.empty else None
            if leader and leader != st.session_state.selected_user:
                st.session_state.selected_user = leader
                # reset graph data when leader changes
                st.session_state.hr_data = pd.DataFrame({
                    'timestamp': pd.Series(dtype='datetime64[ns, UTC]'),
                    'heart_rate': pd.Series(dtype='float64'),
                    'hr_zone': pd.Series(dtype='int64'),
                    'estimated_power': pd.Series(dtype='float64'),
                    'cumulative_calories_burned': pd.Series(dtype='float64')
                })

            return html_table, len(df_display)
    except Exception as e:
        st.error(f"Failed to update leaderboard: {str(e)}")
    return None

# Title (full-width — dropdown removed)
st.title("❤️ Live Heart Rate Monitor")

# Metrics row
metrics_cols = st.columns([1, 1, 1, 1, 0.3])

# Update data and metrics
latest_data = update_live_graph()

# Display metrics
if latest_data is not None:
    metrics_cols[0].metric(
        "Heart Rate", 
        f"{latest_data['heart_rate']} BPM",
        delta=None
    )
    metrics_cols[1].metric(
        "Zone", 
        f"Zone {latest_data['hr_zone']}",
        delta=None
    )
    metrics_cols[2].metric(
        "Power", 
        f"{latest_data['estimated_power']}W",
        delta=None
    )
    metrics_cols[3].metric(
        "Calories", 
        f"{latest_data['cumulative_calories_burned']:.1f} kcal",
        delta=None
    )

    # Leader avatar (top-right)
    avatar_url = get_profile_image(st.session_state.selected_user) if st.session_state.selected_user else None
    if avatar_url:
        metrics_cols[4].markdown(
            f"<img src='{avatar_url}' style='border-radius:50%; width:48px;'>",
            unsafe_allow_html=True,
        )

# Create and update graph
if not st.session_state.hr_data.empty:
    fig = go.Figure()
    
    # Add heart rate trace
    fig.add_trace(go.Scatter(
        x=st.session_state.hr_data['timestamp'],
        y=st.session_state.hr_data['heart_rate'],
        mode='lines+markers',
        name='Heart Rate',
        line=dict(color='#FF4B4B', width=2),
        fill='tozeroy'
    ))
    
    # Add zone lines
    zone_colors = ['rgba(255,255,255,0.3)'] * 4
    zone_labels = ['Zone 1-2', 'Zone 2-3', 'Zone 3-4', 'Zone 4-5']
    zone_values = [120, 140, 160, 180]
    
    for value, label, color in zip(zone_values, zone_labels, zone_colors):
        fig.add_hline(
            y=value,
            line_dash="dash",
            line_color=color,
            annotation_text=label,
            annotation_position="right"
        )
    
    # Update layout
    fig.update_layout(
        xaxis_title="Time",
        yaxis_title="Heart Rate (BPM)",
        showlegend=False,
        height=400,
        margin=dict(l=0, r=0, t=20, b=0),
        plot_bgcolor='rgba(0,0,0,0)',
        paper_bgcolor='rgba(0,0,0,0)',
        xaxis=dict(
            showgrid=True,
            gridcolor='rgba(255,255,255,0.1)',
            range=[
                st.session_state.hr_data['timestamp'].min(),
                st.session_state.hr_data['timestamp'].max()
            ]
        ),
        yaxis=dict(
            showgrid=True,
            gridcolor='rgba(255,255,255,0.1)',
            range=[
                max(0, st.session_state.hr_data['heart_rate'].min() - 10),
                st.session_state.hr_data['heart_rate'].max() + 10
            ]
        )
    )
    
    # Display the graph
    st.plotly_chart(fig, use_container_width=True)
else:
    st.info("No data available yet. Please wait for data to appear.")



# Leaderboard returns HTML string + row count
leaderboard_result = update_leaderboard()
if leaderboard_result is not None:
    leaderboard_html, num_rows = leaderboard_result
    # If we have many rows, constrain the height; otherwise display full table without scrollbar
    ROW_HEIGHT_PX = 35  # approximate row height incl. header padding
    MAX_VISIBLE_ROWS = 10  # only add scrolling if more than this many rows

    if num_rows > MAX_VISIBLE_ROWS:
        table_height = (MAX_VISIBLE_ROWS + 1) * ROW_HEIGHT_PX  # header + visible rows
        st.markdown(
            f"<div style='max-height:{table_height}px; overflow:auto; width:100%'>{leaderboard_html}</div>",
            unsafe_allow_html=True,
        )
    else:
        # No overflow restriction – full height so no scrolling needed for small tables
        st.markdown(leaderboard_html, unsafe_allow_html=True)
else:
    st.info("Loading leaderboard data...")

# Update frequency
time.sleep(1)
st.rerun()

# Footer
st.markdown("---")
st.markdown("""
<div style='text-align: center'>
    Built with Streamlit ❤️ | Monitoring heart rates in real-time
</div>
""", unsafe_allow_html=True)


