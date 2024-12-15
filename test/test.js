import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn, spawnSync } from 'child_process';

import { Builder, By, Capabilities } from 'selenium-webdriver';
import { expect } from 'chai';

// Configuration constants
const CONFIG = {
  TIMEOUTS: {
    // building of the app (5 minutes)
    BUILD: 300000,
    // initialization of the app
    INITIALIZATION: 500,
    // waiting time after new window opening
    WINDOW_WAIT: 1500,
    // waiting time after standard operations
    SHORT_WAIT: 400,
    // waiting time for the inference computation
    INFERENCE_WAIT: 2000,
    // delay before trying to reinitialize
    RETRY_DELAY: 250,
    // how many times try to reinitialize starting driver
    RETRY_ATTEMPTS: 4,
  },
  PATHS: {
    // path to Tauri driver
    TAURI_DRIVER: path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
    // path to Sketchbook binary (you might need to update depending on your OS)
    APPLICATION: path.resolve(
      path.dirname(fileURLToPath(import.meta.url)),
      '..',
      'src-tauri',
      'target',
      'release',
      'Biodivine Sketchbook.exe'
    ),
  },
  // Selenium settings
  SELENIUM: {
    // which port will the driver use
    SERVER_URL: 'http://localhost:4444/',
    BROWSER_NAME: 'wry',
    TAURI_OPTIONS: {
      application: null, // To be set below
      webviewOptions: {},
    },
  },
};

// Assign APPLICATION path to SELENIUM.TAURI_OPTIONS.application
CONFIG.SELENIUM.TAURI_OPTIONS.application = CONFIG.PATHS.APPLICATION;

// Keep track of the WebDriver instance and Tauri driver process
let driver;
let tauriDriver;

// Async wrapper to retry a certain async function multiple times with a small delay.
// This for example makes the resetting/initialization of the driver more robust.
async function retryAsync(fn, retries = 3, delay = 300) {
  let attempts = 0;
  while (attempts < retries) {
    try {
      return await fn();
    } catch (error) {
      attempts++;
      if (attempts >= retries) {
        throw new Error(`Function failed after ${retries} attempts: ${error.message}`);
      }
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }
}

beforeEach(async function () {
  // Set timeout to allow the program to build if needed
  this.timeout(CONFIG.TIMEOUTS.BUILD);

  // Uncomment the following line if you need to build the application before tests
  //spawnSync('cargo', ['tauri', 'build']);

  // Start Tauri driver
  tauriDriver = spawn(CONFIG.PATHS.TAURI_DRIVER, [], { stdio: 'ignore' });

  const capabilities = new Capabilities();
  capabilities.set('tauri:options', CONFIG.SELENIUM.TAURI_OPTIONS);
  capabilities.setBrowserName(CONFIG.SELENIUM.BROWSER_NAME);

  // Start the WebDriver client (if initially fails to connect, try again a few times just in case)
  driver = await retryAsync(
    () => new Builder()
      .withCapabilities(capabilities)
      .usingServer(CONFIG.SELENIUM.SERVER_URL)
      .build(), 
    CONFIG.TIMEOUTS.RETRY_ATTEMPTS,
    CONFIG.TIMEOUTS.RETRY_DELAY
  );
});

afterEach(async function () {
  // Stop the WebDriver session
  await driver.quit()

  // Terminate Tauri driver process
  if (tauriDriver) {
    tauriDriver.kill()
  }
});

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/** Utility to simplify running query selector through shadow root of we-component elems. */
async function findInShadowRoot(element, selector, driver) {
  const script = `
    const shadowRoot = arguments[0].shadowRoot;
    return shadowRoot ? shadowRoot.querySelector(arguments[1]) : null;
  `;
  return driver.executeScript(script, element, selector);
}

/** Utility to open the example model from the signpost page. */
async function openExampleModel(driver, rootComponent) {
    // find and click on button to open example model
    const initialScreenComponent = await findInShadowRoot(rootComponent, "initial-screen", driver);
    const loadExamplebutton = await findInShadowRoot(initialScreenComponent, '#load-example-button', driver);
    
    const loadButtonText = await loadExamplebutton.getText();
    expect(loadButtonText).to.match(/OPEN EXAMPLE/);
    
    await driver.executeScript("arguments[0].click();", loadExamplebutton);
    await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);
}

/** Utility to open a specific tab from the root component. */
async function openTab(driver, rootComponent, tabButtonSelector, expectedText) {
  const navBarComponent = await findInShadowRoot(rootComponent, "nav-bar", driver);
  const tabBarComponent = await findInShadowRoot(navBarComponent, "tab-bar", driver);
  const tabButton = await findInShadowRoot(tabBarComponent, tabButtonSelector, driver);
  const tabButtonText = await tabButton.getText();
  expect(tabButtonText).to.match(new RegExp(expectedText));

  await driver.executeScript("arguments[0].click();", tabButton);
  await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);
}

/** Utility to open a new empty sketch from the signpost page. */
async function openEmptyModel(driver, rootComponent) {
  const initialScreenComponent = await findInShadowRoot(rootComponent, "initial-screen", driver);
  const loadNewSketchbutton = await findInShadowRoot(initialScreenComponent, '#new-sketch-button', driver);
  
  const loadButtonText = await loadNewSketchbutton.getText();
  expect(loadButtonText).to.match(/START NEW SKETCH/);
  
  await driver.executeScript("arguments[0].click();", loadNewSketchbutton);
  await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);
}

describe('Basic walkthrough test', () => {
  /** Very simple test to check if initial window renders correctly. */
  it('should display the welcome message', async () => {
    await sleep(CONFIG.TIMEOUTS.INITIALIZATION);

    const rootComponent = await driver.findElement(By.css("root-component"));
    const initialScreenComponent = await findInShadowRoot(rootComponent, "initial-screen", driver);
    const heading = await findInShadowRoot(initialScreenComponent, "h2", driver);
    
    const headingText = await heading.getText();
    expect(headingText).to.match(/Welcome to SketchBook/);
  });

  /** 
   * Test that user can open example and execute the whole inference.
   * This is the most critical test that ensures the inference session is 
   * created correctly, and that the main workflow is executable.
   */
  it('should complete the example inference walkthrough', async () => {
    await sleep(CONFIG.TIMEOUTS.INITIALIZATION);

    // check we have one window opened
    const originalWindow = await driver.getWindowHandle();
    expect((await driver.getAllWindowHandles()).length).to.equal(1);

    // open example model, then find and click on button to open analysis tab
    const rootComponent = await driver.findElement(By.css("root-component"));
    await openExampleModel(driver, rootComponent)
    await openTab(driver, rootComponent, '#Analysis-tab-button', 'ANALYSIS');
    
    // find and  click on button to open inference window
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#analysis", driver);
    const analysisTab = await findInShadowRoot(contentPaneComponent, "analysis-tab", driver);
    const inferenceButton = await findInShadowRoot(analysisTab, '#open-inference-button', driver);
    const inferenceButtonText = await inferenceButton.getText();
    expect(inferenceButtonText).to.match(/START INFERENCE SESSION/);
    
    await driver.executeScript("arguments[0].click();", inferenceButton);
    // Wait for new window
    await driver.wait(
      async () => (await driver.getAllWindowHandles()).length === 2,
      CONFIG.TIMEOUTS.WINDOW_WAIT
    );
    
    const windows = await driver.getAllWindowHandles();
    windows.forEach(async handle => {
      if (handle !== originalWindow) {
        await driver.switchTo().window(handle);
      }
    });
    
    // find and click button to run computation (only static inference, due to time)
    const analysisComponent = await driver.findElement(By.css("analysis-component"));
    const staticInferenceButton = await findInShadowRoot(analysisComponent, "#static-inference-button", driver);
    const staticInferenceButtonText = await staticInferenceButton.getText();
    expect(staticInferenceButtonText).to.match(/RUN STATIC INFERENCE/);
    await driver.executeScript("arguments[0].click();", staticInferenceButton);
    await sleep(CONFIG.TIMEOUTS.INFERENCE_WAIT);

    // check the results message (should contain correct number of admissible candidates)
    const resultsMessageElem = await findInShadowRoot(analysisComponent, ".overview-message", driver);
    const resultsMessageText = await resultsMessageElem.getText();
    expect(resultsMessageText).to.include("Number of satisfying candidates: 1296");
  });

  /** Test that user can successfully execute the consistency check. */
  it('should perform an example consistency check', async () => {
    await sleep(CONFIG.TIMEOUTS.INITIALIZATION);

    // open example model, then find and click on button to open analysis tab
    const rootComponent = await driver.findElement(By.css("root-component"));
    await openExampleModel(driver, rootComponent)
    await openTab(driver, rootComponent, '#Analysis-tab-button', 'ANALYSIS');

    // find and click on button to run consistency check
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#analysis", driver);
    const analysisTab = await findInShadowRoot(contentPaneComponent, "analysis-tab", driver);
    const consistencyCheckButton = await findInShadowRoot(analysisTab, '#consistency-check-button', driver);
    const consistencyCheckButtonText = await consistencyCheckButton.getText();
    expect(consistencyCheckButtonText).to.match(/RUN CONSISTENCY CHECK/);

    await driver.executeScript("arguments[0].click();", consistencyCheckButton);
    await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);

    // find the text area with the consistency check results
    const consistencyCheckArea = await findInShadowRoot(analysisTab, 'textarea', driver);
    const consistencyCheckAreaText = await consistencyCheckArea.getText();
    expect(consistencyCheckAreaText).to.match(/No issues with the sketch were discovered!/);
  });

  /** Test that user can successfully create new datasets. */
  it('should create a dataset', async () => {
    await sleep(CONFIG.TIMEOUTS.INITIALIZATION);

    // open example model, then find and click on button to open observations tab
    const rootComponent = await driver.findElement(By.css("root-component"));
    await openExampleModel(driver, rootComponent)
    await openTab(driver, rootComponent, '#Observations-tab-button', 'OBSERVATIONS');

    // find and click on button to add dataset
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#observations", driver);
    const observationsTab = await findInShadowRoot(contentPaneComponent, "observations-editor", driver);
    const addDatasetButton = await findInShadowRoot(observationsTab, '.create-button', driver);
    const addDatasetButtonText = await addDatasetButton.getText();
    expect(addDatasetButtonText).to.match(/\+ CREATE/);

    await driver.executeScript("arguments[0].click();", addDatasetButton);
    await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);

    // find the text with dataset's ID
    const datasetContainer = await findInShadowRoot(observationsTab, '#container0', driver);
    const datasetContainerText = await datasetContainer.getText();
    expect(datasetContainerText).to.include("dataset_1");
  });

  /** Test that user can successfully create new functions. */
  it('should create a function', async () => {
    await sleep(CONFIG.TIMEOUTS.INITIALIZATION);

    // open example model, then find and click on button to open functions tab
    const rootComponent = await driver.findElement(By.css("root-component"));
    await openEmptyModel(driver, rootComponent)
    await openTab(driver, rootComponent, '#Functions-tab-button', 'FUNCTIONS');

    // find and click on button to add supplementary function
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#functions", driver);
    const functionsTab = await findInShadowRoot(contentPaneComponent, "functions-editor", driver);
    const addFunctionButton = await findInShadowRoot(functionsTab, '#add-fn-button', driver);
    const addFunctionButtonText = await addFunctionButton.getText();
    expect(addFunctionButtonText).to.match(/\+ ADD/);

    await driver.executeScript("arguments[0].click();", addFunctionButton);
    await sleep(CONFIG.TIMEOUTS.SHORT_WAIT);

    // find the text with dataset's ID
    const functionTile = await findInShadowRoot(functionsTab, 'function-tile', driver);
    const nameField = await findInShadowRoot(functionTile, '#name-field', driver);
    const nameFieldText = await nameField.getAttribute('value');
    expect(nameFieldText).to.include("fn_1");
  });
});
