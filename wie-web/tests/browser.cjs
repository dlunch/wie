const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const webpack = require('webpack');
const { chromium } = require('playwright');
const { PNG } = require('pngjs');

const root = path.resolve(__dirname, '../..');
const output = fs.mkdtempSync(path.join(os.tmpdir(), 'wie-web-presentation-'));
const fixture = path.join(__dirname, 'fixtures');
const sources = fs.readdirSync(fixture, { recursive: true }).filter(name => name.endsWith('.java')).map(name => path.join(fixture, name));
execFileSync('javac', ['--release', '8', '-d', path.join(output, 'classes'), ...sources]);
execFileSync('jar', ['--create', '--file', path.join(output, 'fixture.jar'), '--manifest', path.join(fixture, 'manifest.mf'),
  '-C', path.join(output, 'classes'), 'PresentationMIDlet.class', '-C', path.join(output, 'classes'), 'PresentationMIDlet$Output.class']);

const pixel = (png, x, y) => Array.from(png.data.subarray((y * png.width + x) * 4, (y * png.width + x) * 4 + 4));
const capture = async (page, backend) => backend === 'webgpu'
  ? Buffer.from(await page.locator('canvas').evaluate(canvas => canvas.toDataURL().split(',')[1]), 'base64')
  : page.locator('canvas').screenshot();
const checkPixels = (png, alternate = false) => {
  const scale = Math.min(png.width / 240, png.height / 320);
  const left = (png.width - 240 * scale) / 2;
  const top = (png.height - 320 * scale) / 2;
  const colors = [[255, 0, alternate ? 255 : 0], [0, 255, 0], [0, 0, 255], [64, 64, 64], [128, 128, 128], [255, 255, 255]];
  for (let i = 0; i < colors.length; i++) {
    const actual = pixel(png, Math.floor(left + (i % 3 * 80 + 40) * scale), Math.floor(top + (Math.floor(i / 3) * 160 + 80) * scale));
    assert.ok(colors[i].every((value, channel) => Math.abs(value - actual[channel]) <= 1), `color ${i}: ${actual}`);
    assert.equal(actual[3], 255);
  }
  if (left > 0 || top > 0) assert.deepEqual(pixel(png, 0, 0), [0, 0, 0, 255]);
};

(async () => {
  await new Promise((resolve, reject) => {
    const compiler = webpack({
      mode: 'development', devtool: false, context: root,
      entry: path.join(__dirname, 'browser-entry.ts'),
      output: { path: output, filename: 'test.js' },
      experiments: { asyncWebAssembly: true },
      resolve: {
        extensions: ['.ts', '.js'],
        alias: {
          '@pkg': path.join(root, 'wie-web/pkg/wie_web.js'),
          '@ts': path.join(root, 'wie-web/src/ts'),
          'midi.ts': path.join(root, 'wie-web/src/ts/midi.ts'),
          'panic.ts': path.join(root, 'wie-web/src/ts/panic.ts'),
          'arm-compiler.ts': path.join(root, 'wie-web/src/ts/arm-compiler.ts'),
        },
      },
      module: { rules: [{ test: /\.ts$/, loader: 'ts-loader', options: { configFile: path.join(root, 'wie-web/tsconfig.json'), transpileOnly: true } }] },
    });
    compiler.run((error, stats) => compiler.close(() => {
      if (error || stats.hasErrors()) reject(error ?? new Error(stats.toString({ all: false, errors: true })));
      else resolve();
    }));
  });
  const server = http.createServer((request, response) => {
    if (request.url === '/') {
      response.setHeader('Content-Type', 'text/html');
      response.end('<style>body{margin:0}#player-output{width:240px;height:320px}canvas{width:100%;height:100%;display:block}</style><div id="player-output"></div><script src="/test.js"></script>');
      return;
    }
    const filename = request.url === '/font.ttf' ? path.join(root, 'assets/neodgm.ttf') : path.join(output, path.basename(request.url));
    response.setHeader('Content-Type', filename.endsWith('.wasm') ? 'application/wasm' : 'application/octet-stream');
    fs.createReadStream(filename).on('error', () => { response.statusCode = 404; response.end(); }).pipe(response);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({
      executablePath: process.env.CHROME_PATH, headless: true,
      args: ['--no-sandbox', '--mute-audio', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader',
        '--use-angle=vulkan', '--use-vulkan=swiftshader', '--use-webgpu-adapter=swiftshader', '--enable-features=Vulkan', '--disable-vulkan-surface'],
    });
    for (const backend of ['webgl2', 'webgpu']) {
      for (const dpr of [1, 2]) {
        const page = await browser.newPage({ viewport: { width: 800, height: 700 }, deviceScaleFactor: dpr });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.addInitScript(backend => {
          window.boundContexts = [];
          window.uploads = 0;
          window.fixturePaint = '';
          const log = console.info;
          console.info = (...args) => {
            const text = args.join(' ');
            if (text.includes('presentation-alternate')) window.fixturePaint = 'alternate';
            if (text.includes('presentation-original')) window.fixturePaint = 'original';
            log(...args);
          };
          if (backend === 'webgl2') Object.defineProperty(navigator, 'gpu', { value: undefined });
          if (backend === 'webgpu') {
            const write = GPUQueue.prototype.writeTexture;
            GPUQueue.prototype.writeTexture = function (...args) { window.uploads++; return write.apply(this, args); };
            const request = GPUAdapter.prototype.requestDevice;
            GPUAdapter.prototype.requestDevice = async function (...args) {
              const device = await request.apply(this, args);
              window.device = device;
              return device;
            };
          } else {
            const write = WebGL2RenderingContext.prototype.texSubImage2D;
            WebGL2RenderingContext.prototype.texSubImage2D = function (...args) { window.uploads++; return write.apply(this, args); };
          }
          const original = HTMLCanvasElement.prototype.getContext;
          HTMLCanvasElement.prototype.getContext = function (name, ...args) {
            window.boundContexts.push(name);
            const context = original.call(this, name, ...args);
            if (name === 'webgl2') window.glContext = context;
            return context;
          };
        }, backend);
        await page.goto(`http://127.0.0.1:${server.address().port}`);
        await page.waitForFunction(() => !!window.presentationTest);
        assert.equal(await page.evaluate(() => window.presentationTest.start('nearest', true)), true);
        await page.waitForTimeout(200);
        const initial = PNG.sync.read(await capture(page, backend));
        assert.deepEqual(pixel(initial, initial.width / 2, initial.height / 2), [0, 0, 0, 255]);
        assert.equal(await page.evaluate(() => window.presentationTest.state().ticks), 0);
        await page.evaluate(() => window.presentationTest.stop());
        for (const filter of ['nearest', 'linear']) {
          await page.evaluate(() => { window.fixturePaint = ''; });
          assert.equal(await page.evaluate(filter => window.presentationTest.start(filter), filter), true);
          assert.equal(await page.evaluate(() => document.querySelector('canvas') !== window.previousCanvas), true);
          await page.evaluate(backend => {
            if (backend === 'webgpu') window.previousDevice?.destroy();
            else window.previousGl?.getExtension('WEBGL_lose_context').loseContext();
          }, backend);
          await page.waitForFunction(() => window.fixturePaint === 'original' || window.presentationTest.state().events.some(event => event.type === 'error'));
          await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
          assert.deepEqual(await page.evaluate(() => window.presentationTest.state().events.filter(event => event.type === 'error')), []);
          checkPixels(PNG.sync.read(await capture(page, backend)));
          await page.evaluate(() => { document.getElementById('player-output').style.width = '0px'; });
          await page.waitForTimeout(100);
          const uploads = await page.evaluate(() => window.uploads);
          for (let i = 0; i < 3; i++) {
            await page.evaluate(() => window.presentationTest.key());
            await page.waitForFunction(expected => window.fixturePaint === expected, i % 2 === 0 ? 'alternate' : 'original');
          }
          await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
          assert.equal(await page.evaluate(() => window.uploads), uploads);
          await page.evaluate(() => window.presentationTest.suspend(true));
          const ticks = await page.evaluate(() => window.presentationTest.state().ticks);
          await page.evaluate(() => {
            document.getElementById('player-output').style.width = '333px';
            document.getElementById('player-output').style.height = '270px';
          });
          await page.waitForTimeout(200);
          const screenshot = await capture(page, backend);
          const image = PNG.sync.read(screenshot);
          checkPixels(image, true);
          let blended = false;
          for (let x = 0; x < image.width; x++) {
            const [red, green] = pixel(image, x, Math.floor(image.height / 4));
            if (red > 1 && red < 254 && green > 1 && green < 254) blended = true;
          }
          assert.equal(blended, filter === 'linear');
          assert.equal(await page.evaluate(() => window.uploads), uploads + 1);
          await page.waitForTimeout(200);
          checkPixels(PNG.sync.read(await capture(page, backend)), true);
          assert.equal(await page.evaluate(() => window.presentationTest.state().ticks), ticks);
          assert.deepEqual(await page.evaluate(() => [document.querySelector('canvas').width, document.querySelector('canvas').height]), [333 * dpr, 270 * dpr]);
          if (filter === 'linear') {
            const client = await page.context().newCDPSession(page);
            const changedDpr = dpr === 1 ? 2 : 1;
            // CDP needs a viewport change to dispatch the resolution media-query event.
            await client.send('Emulation.setDeviceMetricsOverride', { width: 801, height: 700, deviceScaleFactor: changedDpr, mobile: false });
            await page.waitForFunction(expected => document.querySelector('canvas').width === expected, 333 * changedDpr);
            checkPixels(PNG.sync.read(await capture(page, backend)), true);
            assert.equal(await page.evaluate(() => window.presentationTest.state().ticks), ticks);
            assert.equal(await page.evaluate(() => window.uploads), uploads + 1);
            await client.detach();
          }
          await page.evaluate(() => {
            window.previousCanvas = document.querySelector('canvas');
            window.previousDevice = window.device;
            window.previousGl = window.glContext;
          });
          await page.evaluate(() => window.presentationTest.stop());
          assert.equal(await page.locator('canvas').count(), 0);
          fs.writeFileSync(path.join(output, `${backend}-${dpr}-${filter}.png`), screenshot);
        }
        await assert.rejects(page.evaluate(() => window.presentationTest.start('cubic')), /Unsupported scale filter/);
        assert.equal(await page.locator('canvas').count(), 0);
        await page.evaluate(() => window.presentationTest.start('nearest', true));
        await page.evaluate(backend => {
          if (backend === 'webgpu') window.device.destroy();
          else window.glContext.getExtension('WEBGL_lose_context').loseContext();
        }, backend);
        await page.waitForFunction(() => window.presentationTest.state().events.some(event => event.type === 'error'));
        assert.equal(await page.locator('canvas').count(), 0);
        await page.evaluate(() => window.presentationTest.stop());
        assert.ok((await page.evaluate(() => window.boundContexts)).includes(backend));
        assert.deepEqual(errors, []);
        console.log(`${backend} DPR=${dpr}: both filters, RGB/alpha/orientation, letterboxing, initial black, idle/paused resize, zero-size frame coalescing, restart, paused GPU loss passed`);
        await page.close();
      }
    }
    console.log(`Screenshots: ${output}`);
  } finally {
    await browser?.close();
    server.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
