/* ============================================
   CMD+K Showcase -- demo.js
   Hero overlay typing animation +
   macOS desktop demo with two animated scenarios
   ============================================ */

(function () {
  'use strict';

  document.addEventListener('DOMContentLoaded', function () {
    // ---- Hero overlay animation (1:1 with demo overlay) ----
    var heroInput = document.getElementById('hero-overlay-input');
    var heroResultEl = document.getElementById('hero-overlay-result');
    var heroDivider = document.querySelector('.overlay-divider');
    var heroTermBody = document.getElementById('hero-terminal-body');

    // Static terminal HTML to restore on reset
    var heroTermStaticHTML =
      '<div class="terminal-line"><span class="prompt">~</span> <span class="cmd">git</span> <span class="flag">log</span> <span class="flag">--oneline</span> <span class="flag">-5</span></div>' +
      '<div class="terminal-line"><span class="hash">c91b934</span> <span class="output">docs: rewrite README with architecture</span></div>' +
      '<div class="terminal-line"><span class="hash">0578c92</span> <span class="output">fix: streaming cursor and a11y styling</span></div>' +
      '<div class="terminal-line"><span class="hash">f964dca</span> <span class="output">fix: Enter-to-confirm and dismiss races</span></div>' +
      '<div class="terminal-line"><span class="hash">d47dd7d</span> <span class="output">feat: real-time token streaming</span></div>' +
      '<div class="terminal-line"><span class="hash">5733218</span> <span class="output">feat: PID-targeted paste & confirm</span></div>' +
      '<div class="terminal-line hero-prompt-line" style="margin-top:8px"><span class="prompt">~</span> <span class="demo-term-cursor"></span></div>';

    var heroScenarios = [
      {
        query: 'find all JS files modified this week',
        tokens: [
          { text: 'find', cls: '' }, { text: ' .', cls: '' },
          { text: ' -name', cls: 'flag' }, { text: ' "*.js"', cls: 'str' },
          { text: ' -mtime', cls: 'flag' }, { text: ' -7', cls: '' },
          { text: ' -type', cls: 'flag' }, { text: ' f', cls: '' }
        ],
        output: ['./src/App.tsx', './src/store/index.ts', './src/components/Overlay.tsx']
      },
      {
        query: 'show disk usage sorted by size',
        tokens: [
          { text: 'du', cls: '' }, { text: ' -sh', cls: 'flag' },
          { text: ' *', cls: '' }, { text: ' | ', cls: '' },
          { text: 'sort', cls: '' }, { text: ' -rh', cls: 'flag' },
          { text: ' | ', cls: '' }, { text: 'head', cls: '' },
          { text: ' -10', cls: 'flag' }
        ],
        output: ['1.2G\tnode_modules', '340M\tdist', '48M\tsrc']
      },
      {
        query: 'list running processes using port 3000',
        tokens: [
          { text: 'lsof', cls: '' }, { text: ' -i', cls: 'flag' },
          { text: ' :3000', cls: '' }
        ],
        output: ['node    1234 user   23u  IPv4  TCP *:3000 (LISTEN)']
      },
      {
        query: 'compress all png images in current folder',
        tokens: [
          { text: 'find', cls: '' }, { text: ' .', cls: '' },
          { text: ' -name', cls: 'flag' }, { text: ' "*.png"', cls: 'str' },
          { text: ' -exec', cls: 'flag' }, { text: ' pngquant', cls: '' },
          { text: ' --force', cls: 'flag' }, { text: ' --ext', cls: 'flag' },
          { text: ' .png', cls: '' }, { text: ' {}', cls: '' },
          { text: ' \\;', cls: '' }
        ],
        output: ['logo.png (68% saved)', 'icon.png (52% saved)', 'hero.png (71% saved)']
      },
      {
        query: 'count lines of code in src folder',
        tokens: [
          { text: 'find', cls: '' }, { text: ' src', cls: '' },
          { text: ' -name', cls: 'flag' }, { text: ' "*.ts"', cls: 'str' },
          { text: ' | ', cls: '' }, { text: 'xargs', cls: '' },
          { text: ' wc', cls: '' }, { text: ' -l', cls: 'flag' }
        ],
        output: ['  142 src/App.tsx', '   89 src/store/index.ts', '  217 src/components/Overlay.tsx', '  448 total']
      }
    ];

    // Hero loop state
    var heroRunning = false;
    var heroPaused = false;
    var heroCancelToken = { cancelled: false };

    function heroEscapeHtml(text) {
      var div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    }

    // Cancellable delay for hero animations
    function heroDelay(ms, token) {
      return new Promise(function (resolve, reject) {
        setTimeout(function () {
          if (token && token.cancelled) { reject('cancelled'); return; }
          resolve();
        }, ms);
      });
    }

    function heroTypeText(element, text, speed, token) {
      return new Promise(function (resolve, reject) {
        var i = 0;
        function step() {
          if (token && token.cancelled) { reject('cancelled'); return; }
          if (i <= text.length) {
            element.innerHTML =
              heroEscapeHtml(text.substring(0, i)) +
              '<span class="demo-cursor"></span>';
            var nextDelay = speed;
            if (i < text.length) {
              var ch = text[i];
              if (ch === ' ') {
                nextDelay = speed + 15 + Math.random() * 25;
              } else if (Math.random() < 0.08) {
                nextDelay = speed + 40 + Math.random() * 60;
              } else {
                nextDelay = speed + Math.random() * 18;
              }
            }
            i++;
            setTimeout(step, nextDelay);
          } else {
            element.innerHTML = heroEscapeHtml(text);
            resolve();
          }
        }
        step();
      });
    }

    function heroStreamTokensDual(overlayEl, termCmdSpan, tokens, speed, token) {
      return new Promise(function (resolve, reject) {
        var i = 0;
        function step() {
          if (token && token.cancelled) { reject('cancelled'); return; }
          if (i < tokens.length) {
            var t = tokens[i];

            // -- Overlay: append styled token + cursor --
            var overlaySpan = document.createElement('span');
            if (t.cls) overlaySpan.className = t.cls;
            overlaySpan.textContent = t.text;
            var oldOverlayCursor = overlayEl.querySelector('.demo-cursor');
            if (oldOverlayCursor) oldOverlayCursor.remove();
            overlayEl.appendChild(overlaySpan);
            var overlayCursor = document.createElement('span');
            overlayCursor.className = 'demo-cursor';
            overlayEl.appendChild(overlayCursor);

            // -- Terminal: append styled token + cursor --
            if (termCmdSpan) {
              var termSpan = document.createElement('span');
              if (t.cls) termSpan.className = t.cls;
              termSpan.textContent = t.text;
              var oldTermCursor = termCmdSpan.querySelector('.demo-term-cursor');
              if (oldTermCursor) oldTermCursor.remove();
              termCmdSpan.appendChild(termSpan);
              var termCursor = document.createElement('span');
              termCursor.className = 'demo-term-cursor';
              termCmdSpan.appendChild(termCursor);
            }

            i++;
            setTimeout(step, speed + Math.random() * 20);
          } else {
            var finalOverlayCursor = overlayEl.querySelector('.demo-cursor');
            if (finalOverlayCursor) finalOverlayCursor.remove();
            if (termCmdSpan) {
              var finalTermCursor = termCmdSpan.querySelector('.demo-term-cursor');
              if (finalTermCursor) finalTermCursor.remove();
            }
            resolve();
          }
        }
        step();
      });
    }

    function resetHeroOverlay() {
      if (heroInput) heroInput.innerHTML = '';
      if (heroResultEl) {
        heroResultEl.innerHTML = '';
        heroResultEl.style.display = 'none';
        heroResultEl.style.animation = '';
      }
      if (heroDivider) {
        heroDivider.style.display = 'none';
        heroDivider.style.animation = '';
      }
    }

    function resetHeroTerminal() {
      if (heroTermBody) {
        heroTermBody.innerHTML = heroTermStaticHTML;
      }
    }

    async function playHeroScenario(scenario, token) {
      // 1. Reset overlay + terminal
      resetHeroOverlay();
      resetHeroTerminal();
      await heroDelay(600, token);

      // 2. Type query with natural feel (30ms base)
      await heroTypeText(heroInput, scenario.query, 30, token);

      // 3. Pause before showing result
      await heroDelay(400, token);

      // 4. Expand divider + result with result-expand animation
      if (heroDivider) {
        heroDivider.style.display = '';
        heroDivider.style.animation = 'result-expand 180ms ease-out forwards';
      }
      if (heroResultEl) {
        heroResultEl.style.display = '';
        heroResultEl.style.animation = 'result-expand 180ms ease-out forwards';
      }

      await heroDelay(200, token);

      // 5. Prepare terminal: remove cursor from prompt line, append cmd span
      var termCmdSpan = null;
      if (heroTermBody) {
        var promptLine = heroTermBody.querySelector('.hero-prompt-line');
        if (promptLine) {
          var oldCursor = promptLine.querySelector('.demo-term-cursor');
          if (oldCursor) oldCursor.remove();
          termCmdSpan = document.createElement('span');
          termCmdSpan.className = 'hero-term-cmd';
          promptLine.appendChild(termCmdSpan);
        }
      }

      // 6. Dual-stream tokens to overlay + terminal (35ms base)
      await heroStreamTokensDual(heroResultEl, termCmdSpan, scenario.tokens, 35, token);

      // 7. Wait before terminal output
      await heroDelay(500, token);

      // 8. Append output lines to terminal, staggered 200ms each
      if (heroTermBody) {
        for (var j = 0; j < scenario.output.length; j++) {
          await heroDelay(200, token);
          var outputLine = document.createElement('div');
          outputLine.className = 'terminal-line';
          var outputSpan = document.createElement('span');
          outputSpan.className = 'output';
          outputSpan.textContent = scenario.output[j];
          outputLine.appendChild(outputSpan);
          heroTermBody.appendChild(outputLine);
        }

        // 9. Append new prompt with blinking cursor
        await heroDelay(200, token);
        var newPrompt = document.createElement('div');
        newPrompt.className = 'terminal-line';
        newPrompt.innerHTML = '<span class="prompt">~</span> <span class="demo-term-cursor"></span>';
        heroTermBody.appendChild(newPrompt);
      }

      // 10. Hold before next scenario
      await heroDelay(3000, token);
    }

    async function heroRunLoop(token) {
      while (!token.cancelled) {
        for (var i = 0; i < heroScenarios.length; i++) {
          if (token.cancelled) return;
          // Wait while paused
          while (heroPaused && !token.cancelled) {
            await heroDelay(200, token);
          }
          try {
            await playHeroScenario(heroScenarios[i], token);
          } catch (e) {
            if (e === 'cancelled') return;
            throw e;
          }
        }
      }
    }

    function startHeroLoop() {
      if (heroRunning) return;
      heroRunning = true;
      heroPaused = false;
      heroCancelToken = { cancelled: false };
      heroRunLoop(heroCancelToken).catch(function () { /* cancelled */ });
    }

    function pauseHeroLoop() {
      heroPaused = true;
    }

    function resumeHeroLoop() {
      heroPaused = false;
    }

    if (heroInput) {
      // Hide divider and result until animation starts
      if (heroDivider) heroDivider.style.display = 'none';
      if (heroResultEl) heroResultEl.style.display = 'none';

      if ('IntersectionObserver' in window) {
        var heroObserver = new IntersectionObserver(
          function (entries) {
            entries.forEach(function (entry) {
              if (entry.isIntersecting) {
                if (!heroRunning) {
                  startHeroLoop();
                } else {
                  resumeHeroLoop();
                }
              } else {
                if (heroRunning) {
                  pauseHeroLoop();
                }
              }
            });
          },
          { threshold: 0.3 }
        );

        var heroSection = document.querySelector('.hero');
        if (heroSection) {
          heroObserver.observe(heroSection);
        }
      } else {
        setTimeout(startHeroLoop, 1000);
      }
    }

    // ---- macOS Menu Bar Live Clock ----
    var clockEl = document.getElementById('demo-menubar-clock');
    if (clockEl) {
      function updateClock() {
        var now = new Date();
        var options = { weekday: 'short', month: 'short', day: 'numeric',
                        hour: '2-digit', minute: '2-digit', hour12: false };
        clockEl.textContent = now.toLocaleString('en-US', options);
      }
      updateClock();
      setInterval(updateClock, 30000);
    }

    // ---- macOS Desktop Demo Animation ----
    var demoDesktop = document.querySelector('.demo-desktop');
    if (!demoDesktop) return;

    var overlay = document.getElementById('demo-overlay');
    var overlayInput = document.getElementById('demo-overlay-input');
    var overlayResultEl = document.getElementById('demo-overlay-result');
    var overlayDivider = overlay ? overlay.querySelector('.demo-overlay-divider') : null;
    var ctxBadge = document.getElementById('demo-ctx-badge');
    var destructiveBadge = document.getElementById('demo-destructive-badge');
    var terminalBody = document.getElementById('demo-terminal-body');

    if (!overlay || !overlayInput || !overlayResultEl || !terminalBody) return;

    var scenarios = [
      {
        query: 'list all ts files changed today',
        tokens: [
          { text: 'find', cls: '' },
          { text: ' .', cls: '' },
          { text: ' -name', cls: 'flag' },
          { text: ' "*.ts"', cls: 'str' },
          { text: ' -mtime', cls: 'flag' },
          { text: ' 0', cls: '' },
          { text: ' -type', cls: 'flag' },
          { text: ' f', cls: '' }
        ],
        terminalOutput: [
          './src/App.tsx',
          './src/store/index.ts',
          './src/components/Overlay.tsx'
        ],
        isDestructive: false,
        contextBadge: 'zsh'
      },
      {
        query: 'remove all git history and start fresh',
        tokens: [
          { text: 'rm', cls: '' },
          { text: ' -rf', cls: 'flag' },
          { text: ' .git', cls: '' },
          { text: ' && ', cls: '' },
          { text: 'git', cls: '' },
          { text: ' init', cls: '' },
          { text: ' && ', cls: '' },
          { text: 'git', cls: '' },
          { text: ' add', cls: '' },
          { text: ' -A', cls: 'flag' },
          { text: ' && ', cls: '' },
          { text: 'git', cls: '' },
          { text: ' commit', cls: '' },
          { text: ' -m', cls: 'flag' },
          { text: ' "initial"', cls: 'str' }
        ],
        terminalOutput: [],
        isDestructive: true,
        contextBadge: 'zsh'
      }
    ];

    var running = false;
    var paused = false;
    var cancelToken = { cancelled: false };

    // Utility: cancellable delay
    function delay(ms, token) {
      return new Promise(function (resolve, reject) {
        setTimeout(function () {
          if (token && token.cancelled) { reject('cancelled'); return; }
          resolve();
        }, ms);
      });
    }

    // Type text character by character with natural feel
    function typeText(element, text, speed, token) {
      return new Promise(function (resolve, reject) {
        var i = 0;
        function step() {
          if (token && token.cancelled) { reject('cancelled'); return; }
          if (i <= text.length) {
            element.innerHTML =
              escapeHtml(text.substring(0, i)) +
              '<span class="demo-cursor"></span>';

            // Natural typing variance
            var nextDelay = speed;
            if (i < text.length) {
              var ch = text[i];
              if (ch === ' ') {
                // Slightly longer pause after space (word boundary)
                nextDelay = speed + 15 + Math.random() * 25;
              } else if (Math.random() < 0.08) {
                // Occasional micro-pause (8% chance -- mimics thinking)
                nextDelay = speed + 40 + Math.random() * 60;
              } else {
                // Normal variance
                nextDelay = speed + Math.random() * 18;
              }
            }

            i++;
            setTimeout(step, nextDelay);
          } else {
            // Remove cursor after done
            element.innerHTML = escapeHtml(text);
            resolve();
          }
        }
        step();
      });
    }

    // Stream tokens into overlay result, and optionally terminal command line
    // Pass null for termCmdSpan to skip terminal streaming (destructive commands)
    function streamTokensDual(overlayEl, termCmdSpan, tokens, speed, token) {
      return new Promise(function (resolve, reject) {
        var i = 0;
        function step() {
          if (token && token.cancelled) { reject('cancelled'); return; }
          if (i < tokens.length) {
            var t = tokens[i];

            // -- Overlay: append styled token + cursor --
            var overlaySpan = document.createElement('span');
            if (t.cls) overlaySpan.className = t.cls;
            overlaySpan.textContent = t.text;
            var oldOverlayCursor = overlayEl.querySelector('.demo-cursor');
            if (oldOverlayCursor) oldOverlayCursor.remove();
            overlayEl.appendChild(overlaySpan);
            var overlayCursor = document.createElement('span');
            overlayCursor.className = 'demo-cursor';
            overlayEl.appendChild(overlayCursor);

            // -- Terminal: append styled token + cursor (only if provided) --
            if (termCmdSpan) {
              var termSpan = document.createElement('span');
              if (t.cls) termSpan.className = t.cls;
              termSpan.textContent = t.text;
              var oldTermCursor = termCmdSpan.querySelector('.demo-term-cursor');
              if (oldTermCursor) oldTermCursor.remove();
              termCmdSpan.appendChild(termSpan);
              var termCursor = document.createElement('span');
              termCursor.className = 'demo-term-cursor';
              termCmdSpan.appendChild(termCursor);
            }

            i++;
            setTimeout(step, speed + Math.random() * 20);
          } else {
            // Remove cursors when streaming done
            var finalOverlayCursor = overlayEl.querySelector('.demo-cursor');
            if (finalOverlayCursor) finalOverlayCursor.remove();
            if (termCmdSpan) {
              var finalTermCursor = termCmdSpan.querySelector('.demo-term-cursor');
              if (finalTermCursor) finalTermCursor.remove();
            }
            resolve();
          }
        }
        step();
      });
    }

    function escapeHtml(text) {
      var div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    }

    function fadeInOverlay() {
      overlay.style.opacity = '';
      overlay.style.pointerEvents = '';
      overlay.classList.remove('exiting');
      overlay.classList.add('entering');
      return new Promise(function (resolve) {
        setTimeout(function () {
          overlay.classList.remove('entering');
          resolve();
        }, 130);
      });
    }

    function fadeOutOverlay() {
      overlay.classList.add('exiting');
      return new Promise(function (resolve) {
        setTimeout(function () {
          overlay.classList.remove('exiting');
          overlay.style.opacity = '0';
          overlay.style.pointerEvents = 'none';
          resolve();
        }, 110);
      });
    }

    function collapseResultSection() {
      if (overlayDivider) {
        overlayDivider.style.display = 'none';
        overlayDivider.style.animation = '';
      }
      overlayResultEl.style.display = 'none';
      overlayResultEl.style.animation = '';
    }

    function expandResultSection() {
      if (overlayDivider) {
        overlayDivider.style.display = '';
        overlayDivider.style.animation = 'result-expand 180ms ease-out forwards';
      }
      overlayResultEl.style.display = '';
      overlayResultEl.style.animation = 'result-expand 180ms ease-out forwards';
    }

    function resetOverlay() {
      overlayInput.innerHTML = '';
      overlayResultEl.innerHTML = '';
      ctxBadge.textContent = '';
      destructiveBadge.style.opacity = '0';
      overlay.style.opacity = '0';
      overlay.style.pointerEvents = 'none';
      overlay.classList.remove('entering', 'exiting');
      collapseResultSection();
    }

    function resetTerminal() {
      terminalBody.innerHTML =
        '<div class="demo-term-line"><span class="prompt">~/projects/app $</span> <span class="demo-term-cursor"></span></div>';
    }

    async function playScenario(scenario, token) {
      // 1. Reset
      resetOverlay();
      resetTerminal();
      await delay(600, token);

      // 2. Show zsh badge immediately (always visible)
      ctxBadge.textContent = scenario.contextBadge;

      // 3. Fade in overlay (input + footer visible, result section collapsed)
      await fadeInOverlay();
      await delay(200, token);

      // 4. Type query in overlay input (50% faster: 30ms base, natural variance)
      await typeText(overlayInput, scenario.query, 30, token);
      await delay(400, token);

      // 5. Expand result section (simulating Enter press)
      expandResultSection();
      await delay(200, token);

      // 6. Prepare terminal command line for streaming (non-destructive only)
      var termCmdSpan = null;
      if (!scenario.isDestructive) {
        var oldTermCursor = terminalBody.querySelector('.demo-term-cursor');
        if (oldTermCursor) oldTermCursor.remove();
        var promptLine = terminalBody.querySelector('.demo-term-line');
        termCmdSpan = document.createElement('span');
        termCmdSpan.className = 'demo-term-cmd';
        promptLine.appendChild(termCmdSpan);
      }

      // 7. Stream tokens (100% faster: 35ms base)
      // Non-destructive: overlay + terminal simultaneously
      // Destructive: overlay only (termCmdSpan is null)
      await streamTokensDual(overlayResultEl, termCmdSpan, scenario.tokens, 35, token);
      await delay(200, token);

      if (scenario.isDestructive) {
        // 8a. Fade in destructive badge
        await delay(300, token);
        destructiveBadge.style.opacity = '1';
        // Hold to show the warning state
        await delay(2500, token);
      } else {
        // 8b. Show terminal output lines staggered
        await delay(500, token);

        for (var j = 0; j < scenario.terminalOutput.length; j++) {
          await delay(200, token);
          var outputLine = document.createElement('div');
          outputLine.className = 'demo-term-line';
          var outputSpan = document.createElement('span');
          outputSpan.className = 'output';
          outputSpan.textContent = scenario.terminalOutput[j];
          outputLine.appendChild(outputSpan);
          terminalBody.appendChild(outputLine);
        }

        // Show new prompt
        await delay(200, token);
        var newPrompt = document.createElement('div');
        newPrompt.className = 'demo-term-line';
        newPrompt.innerHTML = '<span class="prompt">~/projects/app $</span> <span class="demo-term-cursor"></span>';
        terminalBody.appendChild(newPrompt);

        await delay(1500, token);
      }

      // 9. Fade out overlay
      await fadeOutOverlay();
      await delay(1000, token);
    }

    async function runLoop(token) {
      while (!token.cancelled) {
        for (var i = 0; i < scenarios.length; i++) {
          if (token.cancelled) return;
          // Wait while paused
          while (paused && !token.cancelled) {
            await delay(200, token);
          }
          try {
            await playScenario(scenarios[i], token);
          } catch (e) {
            if (e === 'cancelled') return;
            throw e;
          }
        }
      }
    }

    function startDemo() {
      if (running) return;
      running = true;
      paused = false;
      cancelToken = { cancelled: false };
      runLoop(cancelToken).catch(function () { /* cancelled */ });
    }

    function pauseDemo() {
      paused = true;
    }

    function resumeDemo() {
      paused = false;
    }

    function stopDemo() {
      cancelToken.cancelled = true;
      running = false;
      paused = false;
      resetOverlay();
      resetTerminal();
    }

    // IntersectionObserver: start/pause based on visibility
    if ('IntersectionObserver' in window) {
      var demoObserver = new IntersectionObserver(
        function (entries) {
          entries.forEach(function (entry) {
            if (entry.isIntersecting) {
              if (!running) {
                startDemo();
              } else {
                resumeDemo();
              }
            } else {
              if (running) {
                pauseDemo();
              }
            }
          });
        },
        { threshold: 0.2 }
      );

      demoObserver.observe(demoDesktop);
    } else {
      // Fallback: start immediately
      setTimeout(startDemo, 1000);
    }
  });
})();
