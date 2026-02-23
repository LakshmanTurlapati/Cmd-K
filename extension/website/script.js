// Terminal typing animation
document.addEventListener('DOMContentLoaded', () => {
    // Smooth scroll for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });

    // Animate sections on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '0';
                entry.target.style.transform = 'translateY(20px)';

                setTimeout(() => {
                    entry.target.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
                    entry.target.style.opacity = '1';
                    entry.target.style.transform = 'translateY(0)';
                }, 100);

                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    // Observe all sections
    document.querySelectorAll('section').forEach(section => {
        observer.observe(section);
    });

    // Animate feature cards on scroll
    const cardObserver = new IntersectionObserver((entries) => {
        entries.forEach((entry, index) => {
            if (entry.isIntersecting) {
                setTimeout(() => {
                    entry.target.style.opacity = '0';
                    entry.target.style.transform = 'translateY(20px)';
                    entry.target.style.transition = 'opacity 0.5s ease, transform 0.5s ease';

                    setTimeout(() => {
                        entry.target.style.opacity = '1';
                        entry.target.style.transform = 'translateY(0)';
                    }, 50);
                }, index * 100);

                cardObserver.unobserve(entry.target);
            }
        });
    }, observerOptions);

    document.querySelectorAll('.feature-card').forEach(card => {
        cardObserver.observe(card);
    });

    // Add hover effect to buttons
    const buttons = document.querySelectorAll('.btn');
    buttons.forEach(button => {
        button.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-2px) scale(1.02)';
        });

        button.addEventListener('mouseleave', function() {
            this.style.transform = 'translateY(0) scale(1)';
        });
    });

    // Terminal prompt typing effect
    const typingText = document.querySelector('.typing-text');
    if (typingText) {
        const commands = ['cmd+k', 'generate command', 'ai terminal', 'cmd+k'];
        let commandIndex = 0;
        let charIndex = 0;
        let isDeleting = false;

        function type() {
            const currentCommand = commands[commandIndex];

            if (!isDeleting) {
                typingText.textContent = currentCommand.substring(0, charIndex + 1);
                charIndex++;

                if (charIndex === currentCommand.length) {
                    setTimeout(() => { isDeleting = true; }, 2000);
                }
            } else {
                typingText.textContent = currentCommand.substring(0, charIndex - 1);
                charIndex--;

                if (charIndex === 0) {
                    isDeleting = false;
                    commandIndex = (commandIndex + 1) % commands.length;
                }
            }

            const speed = isDeleting ? 50 : 100;
            setTimeout(type, speed);
        }

        // Start typing animation after a brief delay
        setTimeout(type, 1000);
    }

    // Add keyboard shortcut hint
    document.addEventListener('keydown', (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
            e.preventDefault();
            const prompt = document.querySelector('.terminal-prompt');
            if (prompt) {
                prompt.style.transform = 'scale(1.1)';
                setTimeout(() => {
                    prompt.style.transition = 'transform 0.3s ease';
                    prompt.style.transform = 'scale(1)';
                }, 200);
            }
        }
    });

    // Parallax effect on scroll for hero section
    let lastScrollY = window.scrollY;
    window.addEventListener('scroll', () => {
        const scrollY = window.scrollY;
        const hero = document.querySelector('.hero');

        if (hero) {
            const offset = scrollY * 0.5;
            hero.style.transform = `translateY(${offset}px)`;
            hero.style.opacity = Math.max(0, 1 - scrollY / 500);
        }

        lastScrollY = scrollY;
    });

    // Add active state to steps on scroll
    const stepObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.borderColor = 'var(--accent-green)';
                entry.target.style.transform = 'scale(1.02)';

                setTimeout(() => {
                    entry.target.style.transition = 'all 0.3s ease';
                    entry.target.style.transform = 'scale(1)';
                }, 300);
            }
        });
    }, { threshold: 0.5 });

    document.querySelectorAll('.step').forEach(step => {
        stepObserver.observe(step);
    });

    // Console easter egg
    console.log('%cCMD+K Terminal AI', 'font-size: 24px; color: #00ff88; font-weight: bold;');
    console.log('%cInspired by Cursor\'s Terminal Assist', 'font-size: 14px; color: #00d9ff;');
    console.log('%cCreated by Lakshman Turlapati', 'font-size: 14px; color: #bc6ff1;');
    console.log('%cPress CMD+K or Ctrl+K to see the magic!', 'font-size: 12px; color: #8b949e;');
});
