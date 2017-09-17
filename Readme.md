# Fasternet

This is actually just a *markdown* viewer. It was my project for Hack the North 2017.

## Background

This was originally intended to be a prototype of a fast document browser
that rendered with webrender and fetched documents using an efficient compressed
network protocol that required one round trip to fetch a page.

This would allow aggressive prefetching and fast rendering, hopefully hitting
the target of displaying the next page the frame after you clicked a link.

Unfortunately, this was overly ambitious, as I realized an hour or two into
the hackathon. So I scaled back my ambitions to just making a fast markdown viewer.
So that's what this is for now.
