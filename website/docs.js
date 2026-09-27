import Prism from 'prismjs';
import 'prismjs/components/prism-c.js';
import 'prismjs/components/prism-bash.js';
import 'prismjs/themes/prism-tomorrow.css';

document.addEventListener('DOMContentLoaded', () => {
  // Highlight all code blocks manually since we are using ES modules
  Prism.highlightAll();

  const sections = document.querySelectorAll('.docs-content section');
  const navLinks = document.querySelectorAll('.docs-sidebar a');
  const searchInput = document.getElementById('searchInput');
  const sidebarItems = document.querySelectorAll('#sidebarMenu li');

  // Fuzzy Search for Sidebar
  if(searchInput) {
    searchInput.addEventListener('input', (e) => {
      const term = e.target.value.toLowerCase();
      sidebarItems.forEach(item => {
        const text = item.textContent.toLowerCase();
        item.style.display = text.includes(term) ? 'block' : 'none';
      });
    });
  }

  // Scroll Spy
  window.addEventListener('scroll', () => {
    let current = '';
    sections.forEach(section => {
      const sectionTop = section.offsetTop;
      if (scrollY >= sectionTop - 150) {
        current = section.getAttribute('id');
      }
    });

    navLinks.forEach(link => {
      link.classList.remove('active');
      if (link.getAttribute('href') === `#${current}`) {
        link.classList.add('active');
      }
    });
  });
});
