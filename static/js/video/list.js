document.addEventListener("htmx:load", function () {
    checkScroll();
    fetchUserCountry();
});

function scrollTags(direction) {
    const scrollAmount = 200;
    const tagsContainer = document.querySelector('.tags-container');
    if (direction === 'left') {
        tagsContainer.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
    } else if (direction === 'right') {
        tagsContainer.scrollBy({ left: scrollAmount, behavior: 'smooth' });
    }
}

function checkScroll() {
    const tagsContainer = document.querySelector('.tags-container');

    if (tagsContainer == null) {
        return;
    }
    const rightArrowWrapper = document.querySelector('.arrow-wrapper.right');
    const leftArrowWrapper = document.querySelector('.arrow-wrapper.left');

    const scrollLeft = tagsContainer.scrollLeft;
    const maxScrollLeft = tagsContainer.scrollWidth - tagsContainer.clientWidth;

    if (scrollLeft <= 0) {
        leftArrowWrapper.classList.add('hidden');
    } else {
        leftArrowWrapper.classList.remove('hidden');
    }

    if (scrollLeft >= maxScrollLeft) {
        rightArrowWrapper.classList.add('hidden');
    } else {
        rightArrowWrapper.classList.remove('hidden');
    }

    tagsContainer.style.marginLeft = leftArrowWrapper.classList.contains('hidden') ? '0' : '60px';
    tagsContainer.style.marginRight = rightArrowWrapper.classList.contains('hidden') ? '0' : '60px';
}

function toggleTag(checkbox) {
    const tag = checkbox.closest('.tag');
    if (checkbox.checked) {
        tag.classList.add('selected');
    } else {
        tag.classList.remove('selected');
    }
}

async function fetchUserCountry() {
    try {
        const countryNameElement = document.getElementById("country-name");
        if (countryNameElement == null) {
            return;
        }

        const flagElement = document.getElementById("country-flag");

        const cachedCountryData = localStorage.getItem("userCountryData");
        if (cachedCountryData) {
            const { country, countryCode } = JSON.parse(cachedCountryData);

            countryNameElement.textContent = country;

            if (countryCode) {
                flagElement.src = `https://flagcdn.com/w40/${countryCode.toLowerCase()}.png`;
                flagElement.alt = `${country} Flag`;
                flagElement.style.display = "inline";
            } else {
                flagElement.style.display = "none";
            }
            return;
        }

        const response = await fetch('https://get.geojs.io/v1/ip/country.json');
        const data = await response.json();

        const country = data.name || "your location";
        const countryCode = data.country || "";

        countryNameElement.textContent = country;

        if (countryCode) {
            flagElement.src = `https://flagcdn.com/w40/${countryCode.toLowerCase()}.png`;
            flagElement.alt = `${country} Flag`;
            flagElement.style.display = "inline";
        } else {
            flagElement.style.display = "none";
        }

        localStorage.setItem("userCountryData", JSON.stringify({ country, countryCode }));
    } catch (error) {
        console.error("Error fetching country:", error);
        document.getElementById("location-heading").textContent = `Hot Videos in your location`;
    }
}