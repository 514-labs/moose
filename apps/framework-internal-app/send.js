function generateData(amount) {
  const data = [];
  for (let i = 0; i < amount; i++) {
    data.push({
      id: `test-value${i}`,
      test_column_1: "test-value",
      test_column_2: "test-value",
      test_column_3: "test-value",
      test_column_4: "test-value",
      test_column_5: "test-value",
      test_column_6: "test-value",
      test_column_7: "test-value",
      test_column_8: "test-value",
      test_column_9: "test-value",
      test_column_10: "test-value",
      test_column_11: Math.random(),
      test_column_12: Math.random(),
      test_column_13: Math.random(),
      test_column_14: Math.random(),
      test_column_15: Math.random(),
      test_column_16: Math.random(),
      test_column_17: Math.random(),
      test_column_18: Math.random(),
      test_column_19: Math.random(),
      test_column_20: Math.random(),
      test_column_21: Math.random(),
      test_column_22: Math.random(),
      test_column_23: Math.random(),
      test_column_24: Math.random(),
      test_column_25: Math.random(),
      test_column_26: Math.random(),
      test_column_27: Math.random(),
      test_column_28: Math.random(),
      test_column_29: Math.random(),
      test_column_30: Math.random(),
      test_column_31: Math.random(),
      test_column_32: Math.random(),
      test_column_33: Math.random(),
      test_column_34: Math.random(),
      test_column_35: Math.random(),
      test_column_36: Math.random(),
      test_column_37: Math.random(),
      test_column_38: Math.random(),
      test_column_39: Math.random(),
      test_column_40: Math.random(),
      test_column_41: Math.random(),
      test_column_42: Math.random(),
      test_column_43: Math.random(),
      test_column_44: Math.random(),
      test_column_45: Math.random(),
      test_column_46: Math.random(),
      test_column_47: Math.random(),
      test_column_48: Math.random(),
      test_column_49: Math.random(),
      test_column_50: Math.random(),
      test_column_51: Math.random(),
      test_column_52: Math.random(),
      test_column_53: Math.random(),
      test_column_54: Math.random(),
      test_column_55: Math.random(),
      test_column_56: Math.random(),
      test_column_57: Math.random(),
      test_column_58: Math.random(),
      test_column_59: Math.random(),
      test_column_60: Math.random(),
      test_column_61: Math.random(),
      test_column_62: Math.random(),
    });
  }
  return data;
}

async function sendPostRequest(dataArray) {
  const url = "https://moosefood.514.dev/ingest/BigBatch/0.5";
  // const url = "http://localhost:4000/ingest/BigBatch/0.5";
  const requestOptions = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(dataArray),
  };

  try {
    console.time("fetchTime");
    const response = await fetch(url, requestOptions);
    console.timeEnd("fetchTime");
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
  } catch (error) {
    console.error("Error:", error);
  }
}

const data = generateData(1000);
sendPostRequest(data);
