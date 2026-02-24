/* ============================================
   CMD+K Showcase -- main.js
   Nav scroll, mobile menu, scroll reveal,
   theme toggle, smooth scroll
   ============================================ */

(function () {
  'use strict';

  // ---- Theme ----
  const STORAGE_KEY = 'cmdk-showcase-theme';

  function getPreferredTheme() {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) return stored;
    return 'dark'; // default dark
  }

  function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem(STORAGE_KEY, theme);
  }

  applyTheme(getPreferredTheme());

  document.addEventListener('DOMContentLoaded', function () {
    // ---- Theme toggle ----
    const toggle = document.querySelector('.theme-toggle');
    if (toggle) {
      toggle.addEventListener('click', function () {
        const current = document.documentElement.getAttribute('data-theme');
        applyTheme(current === 'dark' ? 'light' : 'dark');
      });
    }

    // ---- Nav scroll effect ----
    const nav = document.querySelector('.nav');
    if (nav) {
      function checkScroll() {
        if (window.scrollY > 20) {
          nav.classList.add('scrolled');
        } else {
          nav.classList.remove('scrolled');
        }
      }
      window.addEventListener('scroll', checkScroll, { passive: true });
      checkScroll();
    }

    // ---- Mobile menu ----
    const hamburger = document.querySelector('.hamburger');
    const navLinks = document.querySelector('.nav-links');
    const navOverlay = document.querySelector('.nav-overlay');

    function closeMenu() {
      if (hamburger) hamburger.classList.remove('open');
      if (navLinks) navLinks.classList.remove('open');
      if (navOverlay) navOverlay.classList.remove('open');
      document.body.style.overflow = '';
    }

    function openMenu() {
      if (hamburger) hamburger.classList.add('open');
      if (navLinks) navLinks.classList.add('open');
      if (navOverlay) navOverlay.classList.add('open');
      document.body.style.overflow = 'hidden';
    }

    if (hamburger) {
      hamburger.addEventListener('click', function () {
        if (navLinks && navLinks.classList.contains('open')) {
          closeMenu();
        } else {
          openMenu();
        }
      });
    }

    if (navOverlay) {
      navOverlay.addEventListener('click', closeMenu);
    }

    // Close menu on nav link click
    if (navLinks) {
      navLinks.querySelectorAll('a').forEach(function (link) {
        link.addEventListener('click', closeMenu);
      });
    }

    // ---- Smooth scroll for anchor links ----
    document.querySelectorAll('a[href^="#"]').forEach(function (anchor) {
      anchor.addEventListener('click', function (e) {
        var href = this.getAttribute('href');
        if (href === '#') return;
        var target = document.querySelector(href);
        if (target) {
          e.preventDefault();
          target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
      });
    });

    // ---- Scroll reveal ----
    var reveals = document.querySelectorAll('.reveal');
    if (reveals.length > 0 && 'IntersectionObserver' in window) {
      var revealObserver = new IntersectionObserver(
        function (entries) {
          entries.forEach(function (entry) {
            if (entry.isIntersecting) {
              entry.target.classList.add('visible');
              revealObserver.unobserve(entry.target);
            }
          });
        },
        { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
      );

      reveals.forEach(function (el) {
        revealObserver.observe(el);
      });
    } else {
      // Fallback: show all
      reveals.forEach(function (el) {
        el.classList.add('visible');
      });
    }

    // ---- Carousel dynamic width ----
    function setupCarousel() {
      var track = document.querySelector('.carousel-track');
      if (!track) return;
      var items = track.querySelectorAll('.terminal-tag:not([aria-hidden])');
      if (items.length === 0) return;
      var totalWidth = 0;
      items.forEach(function (item) {
        totalWidth += item.offsetWidth;
      });
      // Account for gaps (10px each) between original items
      var gap = 10;
      var offset = totalWidth + (items.length * gap);
      track.style.setProperty('--carousel-offset', '-' + offset + 'px');
      // Speed: ~50px/sec
      var duration = offset / 50;
      track.style.setProperty('--carousel-duration', duration + 's');
    }

    setupCarousel();

    var carouselResizeTimer;
    window.addEventListener('resize', function () {
      clearTimeout(carouselResizeTimer);
      carouselResizeTimer = setTimeout(setupCarousel, 200);
    });

    // ---- Active nav link on scroll ----
    var sections = document.querySelectorAll('section[id]');
    var navAnchors = document.querySelectorAll('.nav-links a[href^="#"]');

    if (sections.length > 0 && navAnchors.length > 0) {
      var sectionObserver = new IntersectionObserver(
        function (entries) {
          entries.forEach(function (entry) {
            if (entry.isIntersecting) {
              var id = entry.target.getAttribute('id');
              navAnchors.forEach(function (a) {
                a.classList.remove('active');
                if (a.getAttribute('href') === '#' + id) {
                  a.classList.add('active');
                }
              });
            }
          });
        },
        { threshold: 0.2, rootMargin: '-64px 0px -50% 0px' }
      );

      sections.forEach(function (section) {
        sectionObserver.observe(section);
      });
    }
  });
})();
