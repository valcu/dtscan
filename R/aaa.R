#' dtscan: Triangle‑Based Density Clustering of 2D Points
#'
#' The dtscan package provides R bindings to a high‑performance Rust
#' implementation of triangle‑based dtscan via Delaunay triangulation.
#'
#' Clusters emerge by iteratively linking points whose connecting edges fall
#' below a user‑defined distance threshold, effectively applying a
#' dtscan‑style density rule on the triangulation graph.
#'
#' @keywords internal
#' @useDynLib dtscan, .registration = TRUE
"_PACKAGE"