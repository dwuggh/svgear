import express from 'express';
import mjAPI from 'mathjax-node';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import * as readline from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';
import { createInterface } from 'node:readline';

// --- Configuration ---
const DEFAULT_PORT = 3000;
const allowedFormats = ['TeX', 'MathML', 'AsciiMath'];

// --- Initialization ---
mjAPI.start();

// --- Command-Line Argument Parsing ---
const argv = yargs(hideBin(process.argv))
  .command('serve', 'Run as a web server', (yargs) => {
    return yargs
      .option('port', {
        describe: 'Server port',
        type: 'number',
        default: DEFAULT_PORT,
      });
  })
  .command('convert', 'Convert a single equation', (yargs) => {
    return yargs
      .option('input', {
        alias: 'i',
        describe: 'Input equation (from string)',
        type: 'string',
      })
      .option('format', {
        alias: 'f',
        describe: 'Input format (TeX, MathML, AsciiMath)',
        type: 'string',
        default: 'TeX',
        choices: allowedFormats,
      })
      .option('output', {
        alias: 'o',
        describe: 'Output file (defaults to stdout)',
        type: 'string',
      });
  })
  .command('stdio', 'Run in JSON-RPC mode over stdio', () => {})
  .demandCommand(1, 'You must specify a command: serve, convert, or stdio')
  .help()
  .argv;

// --- Main Logic ---

async function convertEquation(equation, format, inline = false) {
  try {
    const data = await mjAPI.typeset({
      math: equation,
      format: format,
      svg: true,
      display: !inline, // Use display mode when not inline
    });

    if (!data.errors) {
      return data.svg;
    } else {
      throw new Error(`MathJax error: ${data.errors.join(', ')}`);
    }
  } catch (error) {
    console.error("Error during MathJax processing:", error);
    throw error; // Re-throw to handle in caller
  }
}

// Handle JSON-RPC style requests
async function handleJsonRpcRequest(request) {
  const { id, method, params } = request;
  
  let result = null;
  let error = null;
  
  try {
    switch (method) {
      case 'Paint':
        if (!params.content) {
          throw new Error('Content parameter is required');
        }
        
        const format = params.format || 'TeX';
        if (!allowedFormats.includes(format)) {
          throw new Error(`Invalid format. Supported formats: ${allowedFormats.join(', ')}`);
        }
        
        const svg = await convertEquation(params.content, format);
        result = { svg };
        break;
        
      case 'RenderToBitmap':
        if (!params.paint_params || !params.paint_params.content) {
          throw new Error('paint_params.content is required');
        }
        
        const paintFormat = params.paint_params.format || 'TeX';
        if (!allowedFormats.includes(paintFormat)) {
          throw new Error(`Invalid format. Supported formats: ${allowedFormats.join(', ')}`);
        }
        
        // First convert to SVG
        const svgContent = await convertEquation(params.paint_params.content, paintFormat);
        
        // We don't actually render to bitmap here, just return the SVG
        // This is a placeholder for compatibility with the Rust API
        result = {
          id: generateId(),
          bitmap: {
            data: Buffer.from(svgContent).toString('base64'),
            width: params.width || 800,
            height: params.height || 600
          }
        };
        break;
        
      default:
        throw new Error(`Unknown method: ${method}`);
    }
  } catch (e) {
    error = {
      code: -32000,
      message: e.message
    };
  }
  
  return {
    jsonrpc: '2.0',
    id,
    result,
    error
  };
}

// Generate a simple ID for SVGs
function generateId() {
  return Math.random().toString(36).substring(2, 15);
}

// Run in stdio mode
async function runStdioMode() {
  console.error('Running in stdio mode');
  console.error('Send requests in format: {inline: boolean, content: string}');
  
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
  });
  
  rl.on('line', async (line) => {
    try {
      const request = JSON.parse(line);
      const svg = await processMathJaxRequest(request);
      
      // Send SVG as response
      console.log(svg);
    } catch (e) {
      // Handle errors
      console.error(`Error: ${e.message}`);
      process.exit(1);
    }
  });
  
  // Keep running until stdin closes
  rl.on('close', () => {
    process.exit(0);
  });
}

// Run web server
async function runServer(port) {
  const app = express();
  app.use(express.json());

  // Legacy endpoint for backward compatibility
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

  // New RPC endpoint that matches the Rust API
  app.post('/rpc', async (req, res) => {
    try {
      const response = await handleJsonRpcRequest(req.body);
      res.json(response);
    } catch (error) {
      res.status(500).json({
        jsonrpc: '2.0',
        error: {
          code: -32000,
          message: error.message
        },
        id: req.body.id
      });
    }
  });

  // Global Error Handler
  app.use((err, req, res, next) => {
    console.error(err.stack);
    res.status(500).send('Something broke!');
  });

  app.listen(port, () => {
    console.log(`Server is running on http://localhost:${port}`);
    console.log(`RPC endpoint available at http://localhost:${port}/rpc`);
  });
}

// Run single conversion
async function runConvert() {
  let equation = argv.input;
  const format = argv.format;
  const outputFile = argv.output;

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

  try {
    const svg = await convertEquation(equation, format);
    
    if (outputFile) {
      const fs = await import('fs');
      fs.writeFileSync(outputFile, svg);
      console.error(`SVG written to ${outputFile}`);
    } else {
      console.log(svg);
    }
  } catch (error) {
    console.error("Conversion failed:", error.message);
    process.exit(1);
  }
}

// Main entry point
async function main() {
  const command = argv._[0];
  
  try {
    switch (command) {
      case 'serve':
        await runServer(argv.port);
        break;
        
      case 'convert':
        await runConvert();
        break;
        
      case 'stdio':
        await runStdioMode();
        break;
    }
  } catch (error) {
    console.error("Fatal error:", error);
    process.exit(1);
  }
}

main();
