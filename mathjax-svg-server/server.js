import express from 'express';
import mjAPI from 'mathjax-node';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import * as readline from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

// --- Configuration ---
const DEFAULT_PORT = 3000;
const allowedFormats = ['TeX', 'MathML', 'AsciiMath'];

// --- Initialization ---
mjAPI.start();

// --- Command-Line Argument Parsing ---
const argv = yargs(hideBin(process.argv))
  .option('server', {
    describe: 'Run as a web server',
    type: 'boolean',
    default: false,
  })
  .option('port', {
    describe: 'server port',
    default: DEFAULT_PORT,
  })
  .option('input', {
    alias: 'i',
    describe: 'Input equation (from string)',
    type: 'string',
  })
  .option('format', {
    alias: 'f',
    describe: 'Input format (TeX, MathML, AsciiMath)',
    type: 'string',
    default: 'TeX', // Default format
    choices: allowedFormats, // Validate format
  })
  .help()
  .argv;

// --- Main Logic (Command-Line or Server) ---

async function convertEquation(equation, format) {
  try {
    const data = await mjAPI.typeset({
      math: equation,
      format: format,
      svg: true,
    });

    if (!data.errors) {
      return data.svg;
    } else {
      throw new Error(`MathJax error: ${data.errors.join(', ')}`);
    }
  } catch (error) {
    console.error("Error during MathJax processing:", error);
    process.exit(1); // Exit with an error code
  }
}

async function main() {
  if (argv.server) {
    // --- Server Mode ---
    const port = argv.port;
    const app = express();
    app.use(express.json());

    app.post('/convert', async (req, res) => {
      const { equation, format } = req.body;

      if (!equation) {
        return res.status(400).json({ error: 'Equation is required' });
      }

      if (!allowedFormats.includes(format)) {
        return res.status(400).json({ error: `Invalid format. Supported formats: ${allowedFormats.join(', ')}` });
      }

      try {
        const svg = await convertEquation(equation, format);
        res.set('Content-Type', 'image/svg+xml');
        res.send(svg);
      } catch (error) {
        res.status(500).json({ error: 'Internal Server Error', details: error.message });
      }
    });

    // Global Error Handler
    app.use((err, req, res, next) => {
      console.error(err.stack);
      res.status(500).send('Something broke!');
    });

    app.listen(port, () => {
      console.log(`Server is running on http://localhost:${port}`);
    });

  } else {
    // --- Command-Line Mode ---

    let equation = argv.input;
    const format = argv.format;

    if (!equation) {
      // Read from stdin if no --input
      const rl = readline.createInterface({ input, output });
      
      try {
          let lines = [];
          for await (const line of rl) {
              lines.push(line);
          }
          equation = lines.join('\n');  // Join lines with newline
          if (!equation) {
            console.error("Error: No equation provided via --input or stdin.");
              process.exit(1);
          }
      } 
      finally {
        rl.close(); // Always close the readline interface
      }
    }

    const svg = await convertEquation(equation, format);
    console.log(svg);
  }
}

main();