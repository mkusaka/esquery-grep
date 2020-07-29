const doc = document.createElement("a");

doc.addEventListener("click", (e) => {
  console.log(e.currentTarget.href);
});
