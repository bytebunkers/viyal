export async function updateVersionFromGithub(repo) {
  try {
    const response = await fetch(`https://api.github.com/repos/${repo}/releases/latest`);
    if (!response.ok) return;
    const release = await response.json();
    const version = release.tag_name;

    // Update text content for any element with class 'github-version'
    document.querySelectorAll('.github-version').forEach(el => {
      el.textContent = el.textContent.replace(/v\d+\.\d+\.\d+([-\w.]+)?/g, version);
    });

    // Update href for any element with class 'github-release-link'
    document.querySelectorAll('.github-release-link').forEach(link => {
       link.href = release.html_url;
    });

    // Windows specific download link
    const winDownload = document.querySelector('.github-windows-download');
    if (winDownload) {
      const winAsset = release.assets?.find(a => a.name.toLowerCase().includes('windows') && a.name.endsWith('.zip'));
      if (winAsset) {
        winDownload.href = winAsset.browser_download_url;
      } else {
        winDownload.href = release.html_url; // fallback to release page
      }
    }
  } catch (err) {
    console.error('Failed to fetch GitHub release:', err);
  }
}
