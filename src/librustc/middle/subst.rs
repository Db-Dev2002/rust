// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Type substitutions.

use core::prelude::*;
use middle::ty;
use util::ppaux::Repr;

///////////////////////////////////////////////////////////////////////////
// Public trait `Subst`
//
// Just call `foo.subst(tcx, substs)` to perform a substitution across
// `foo`.

pub trait Subst {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> Self;
}

///////////////////////////////////////////////////////////////////////////
// Substitution over types
//
// Because this is so common, we make a special optimization to avoid
// doing anything is `substs` is a no-op.  I tried to generalize these
// to all subst methods but ran into trouble due to the limitations of
// our current method/trait matching algorithm. - Niko

trait Subst1 {
    fn subst1(&self, tcx: ty::ctxt, substs: &ty::substs) -> Self;
}

impl Subst for ty::t {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::t {
        if ty::substs_is_noop(substs) {
            return *self;
        } else {
            return self.subst1(tcx, substs);
        }
    }
}

impl Subst1 for ty::t {
    fn subst1(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::t {
        if !ty::type_needs_subst(*self) {
            return *self;
        }

        match ty::get(*self).sty {
            ty::ty_param(p) => substs.tps[p.idx],
            ty::ty_self(_) => substs.self_ty.get(),
            _ => {
                ty::fold_regions_and_ty(
                    tcx, *self,
                    |r| match r {
                        ty::re_bound(ty::br_self) => {
                            match substs.self_r {
                                None => {
                                    tcx.sess.bug(
                                        fmt!("ty::subst: \
                                              Reference to self region when \
                                              given substs with no self region, \
                                              ty = %s",
                                             self.repr(tcx)));
                                }
                                Some(self_r) => self_r
                            }
                        }
                        _ => r
                    },
                    |t| t.subst1(tcx, substs),
                    |t| t.subst1(tcx, substs))
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////
// Other types

impl<T:Subst> Subst for ~[T] {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ~[T] {
        self.map(|t| t.subst(tcx, substs))
    }
}

impl<T:Subst> Subst for @~[T] {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> @~[T] {
        @(**self).subst(tcx, substs)
    }
}

impl<T:Subst> Subst for Option<T> {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> Option<T> {
        self.map(|t| t.subst(tcx, substs))
    }
}

impl Subst for ty::TraitRef {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::TraitRef {
        ty::TraitRef {
            def_id: self.def_id,
            substs: self.substs.subst(tcx, substs)
        }
    }
}

impl Subst for ty::substs {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::substs {
        ty::substs {
            self_r: self.self_r,
            self_ty: self.self_ty.map(|typ| typ.subst(tcx, substs)),
            tps: self.tps.map(|typ| typ.subst(tcx, substs))
        }
    }
}

impl Subst for ty::BareFnTy {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::BareFnTy {
        ty::fold_bare_fn_ty(self, |t| t.subst(tcx, substs))
    }
}

impl Subst for ty::param_bound {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::param_bound {
        match self {
            &ty::bound_copy |
            &ty::bound_durable |
            &ty::bound_owned |
            &ty::bound_const => {
                *self
            }

            &ty::bound_trait(tref) => {
                ty::bound_trait(@tref.subst(tcx, substs))
            }
        }
    }
}

impl Subst for ty::TypeParameterDef {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::TypeParameterDef {
        ty::TypeParameterDef {
            def_id: self.def_id,
            bounds: self.bounds.subst(tcx, substs)
        }
    }
}

impl Subst for ty::Generics {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::Generics {
        ty::Generics {
            type_param_defs: self.type_param_defs.subst(tcx, substs),
            region_param: self.region_param
        }
    }
}

impl Subst for ty::ty_param_bounds_and_ty {
    fn subst(&self, tcx: ty::ctxt, substs: &ty::substs) -> ty::ty_param_bounds_and_ty {
        ty::ty_param_bounds_and_ty {
            generics: self.generics.subst(tcx, substs),
            ty: self.ty.subst(tcx, substs)
        }
    }
}

