# Poetry-Identification
Poetry Identification Code from my dissertation runs on zip files containing DJVUXML from the Internet Archive.

## Where did this model come from?

For details about where this model came from, or what it does, refer to [my dissertation](https://scholarworks.umass.edu/dissertations_2/1573/) for now.

```bibtex
@phdthesis{foley2019thesis,
  author = {John Foley},
  title = {{Poetry: Identification, Entity Recognition, and Retrieval}},
  year = {2019},
  school = {University of Massachusetts},
}
```

## Can I get some data for this?

Data from my dissertation is available at [CIIR/downloads/poetry](http://ciir.cs.umass.edu/downloads/poetry). The training data used to build the model is there, as well as the output of this model on the 50,000 books from the INEX 2007 challenge (basically a random sample of Internet Archive books).

## About this Code

This code is written in Rust. There are two packages: ``djvuxml-rs`` which is a pretty generic way to interact with internet-archive scanned book files, and ``classification`` which runs through using a JSONified Random Forest model and makes predictions at the page level. The files on [CIIR/downloads/poetry](http://ciir.cs.umass.edu/downloads/poetry) -- Poetry50K collection were generated from de-duplicating the output of this code.

## Help? Where's the code for XXX?

I'm slowly cleaning up and open-sourcing all the code. If you're looking for a piece that's not made it public yet, please don't hesitate to contact me! File an issue here or check out my [personal website](https://jjfoley.me) to find my latest academic email.
