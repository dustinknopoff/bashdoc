function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
let cards = Array.from(document.getElementsByClassName("card"));
let as = Array.from(document.getElementsByTagName("a"));
as.map(tag => tag.addEventListener("click", highlight));
function highlight(event) {
  let id = event.srcElement.hash;
  cards.map(card => {
    if (`#${card.id}` === id) {
      let { backgroundColor, width, ...rest } = card.style;
      card.style.backgroundColor = "#efefef";
      card.style.width = "35%";
      sleep(1000).then(() => {
        card.style.backgroundColor = backgroundColor;
        card.style.width = width;
      });
    }
  });
}
