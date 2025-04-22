#' Triangle-Based dtscan Clustering
#'
#' @description
#' Perform density-based clustering on a 2D point set via a triangle‑based algorithm
#' implemented in Rust for high performance. A Delaunay triangulation is computed,
#' edges are extracted and sorted by length, and clusters grow by traversing short edges
#' from core points.
#'
#' @param x  
#'   Numeric matrix (n × 2) or two‑column data.frame of coordinates (x, y).
#' @param min_pts  
#'   Integer; minimum number of neighbors for a point to be considered a core point.
#'   Defaults to `5L`.
#' @param max_closeness  
#'   Numeric; maximum edge length for two points to be considered connected.
#' @param parallel  
#'   Logical; whether to preprocess the triangulation in parallel.
#'   Defaults to `TRUE`.
#'
#' @return  
#' Integer vector of length `nrow(x)`, giving 1‑based cluster IDs. Points not
#' assigned to any cluster are `NA`-s.
#'
#' @note  
#' - Duplicate or collinear points can produce degenerate triangles; ensure `x` has unique,  
#'   non‑collinear coordinates.  
#' - Preprocessing in parallel (done internally by Rust) can speed up large point sets
#'   but may incur overhead on small data;  
#' - Unlike standard dtscan, dtscan uses the Delaunay graph for neighborhood definition, so  
#'   cluster boundaries may differ from other implementations. 
#'
#' @references
#' Kim, S. & Cho, Y. (2019). Triangle-based density clustering using Delaunay triangulation.  
#' Sensors, 19(18), 3926.  
#' \url{https://pmc.ncbi.nlm.nih.gov/articles/PMC6767241/}  
#'  
#' Ported from \url{https://github.com/randogoth/xenobalanus}
#'
#'
#' @examples
#' data("penguins")
#' xy = penguins[, c('bill_len', 'bill_dep')] |> na.omit()
#' xy$clust_id = dtscan(xy, max_closeness = 0.85)
#' xy$col <- ifelse(is.na(xy$clust_id), "lightgrey", 
#'  hcl.colors(length(u <- sort(unique(na.omit(xy$clust_id)))))[match(xy$clust_id, u)])
#' plot(xy[, c(1,2)], col = xy$col, pch = 19)
#' 
#' @export

dtscan <-  function(x, min_pts = 5, max_closeness ,parallel = TRUE) {

  stopifnot(!anyNA(x))

  dt <- new_dtscan()
  dtscan_set_points(dt, x[, 1], x[, 2])
  dtscan_delaunay(dt)
  dtscan_preprocess(dt, 0L, parallel)
  clusters <- dtscan_run(dt, as.integer(min_pts), max_closeness)

  cluster_id <- rep(0L, nrow(x))
  for (i in seq_along(clusters)) {
    cluster_id[clusters[[i]] + 1] <- i  # convert from 0-based to 1-based (+1)
  }

  cluster_id
}