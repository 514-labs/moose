const [, , , SCRIPTS_DIR_PATH] = process.argv;

export const runScripts = async () => {
  console.log(`Starting scripts runner for directory: ${SCRIPTS_DIR_PATH}`);
};
