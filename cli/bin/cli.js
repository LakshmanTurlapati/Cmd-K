#!/usr/bin/env node

const { spawn, exec } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

const args = process.argv.slice(2);
const cmd = args[0];

const ELECTRON_PATH = path.join(__dirname, '..', 'node_modules', '.bin', 'electron');
const MAIN_PATH = path.join(__dirname, '..', 'main.js');
const PID_FILE = path.join(os.homedir(), '.cmdk.pid');

function start() {
  // Check if already running
  if (fs.existsSync(PID_FILE)) {
    const pid = fs.readFileSync(PID_FILE, 'utf8').trim();
    try {
      process.kill(parseInt(pid), 0);
      console.log('cmdk is already running (PID: ' + pid + ')');
      return;
    } catch (e) {
      // Process not running, clean up stale PID file
      fs.unlinkSync(PID_FILE);
    }
  }

  // Start Electron in background
  const child = spawn(ELECTRON_PATH, [MAIN_PATH], {
    detached: true,
    stdio: 'ignore',
    cwd: path.join(__dirname, '..'),  // Use package directory as cwd
    env: { ...process.env, ELECTRON_RUN_AS_NODE: '' }
  });

  child.unref();
  fs.writeFileSync(PID_FILE, child.pid.toString());
  console.log('cmdk started (PID: ' + child.pid + ')');
  console.log('Press CMD+K to show overlay');
}

function stop() {
  if (!fs.existsSync(PID_FILE)) {
    console.log('cmdk is not running');
    return;
  }

  const pid = fs.readFileSync(PID_FILE, 'utf8').trim();
  try {
    process.kill(parseInt(pid), 'SIGTERM');
    fs.unlinkSync(PID_FILE);
    console.log('cmdk stopped');
  } catch (e) {
    fs.unlinkSync(PID_FILE);
    console.log('cmdk was not running (cleaned up stale PID file)');
  }
}

function status() {
  if (!fs.existsSync(PID_FILE)) {
    console.log('cmdk is not running');
    return;
  }

  const pid = fs.readFileSync(PID_FILE, 'utf8').trim();
  try {
    process.kill(parseInt(pid), 0);
    console.log('cmdk is running (PID: ' + pid + ')');
  } catch (e) {
    fs.unlinkSync(PID_FILE);
    console.log('cmdk is not running (cleaned up stale PID file)');
  }
}

function help() {
  console.log(`
cmdk - AI-powered command generation

Usage:
  cmdk start    Start the background daemon
  cmdk stop     Stop the daemon
  cmdk status   Check if daemon is running
  cmdk help     Show this help

Once started, press CMD+K to show the overlay.
`);
}

switch (cmd) {
  case 'start':
    start();
    break;
  case 'stop':
    stop();
    break;
  case 'status':
    status();
    break;
  case 'help':
  case '--help':
  case '-h':
    help();
    break;
  default:
    if (cmd) {
      console.log('Unknown command: ' + cmd);
    }
    help();
}
