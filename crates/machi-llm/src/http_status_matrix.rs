// W6: HTTP status → HttpRetryClass contract matrix (one test per status).
#[cfg(test)]
#[allow(clippy::missing_assert_message, reason = "matrix cases use status in name")]
mod http_status_matrix {
    use super::{HttpRetryClass, classify_http_status};

    fn expected(status: u16) -> HttpRetryClass {
        match status {
            400 | 401 | 403 | 404 | 422 | 525 | 526 => HttpRetryClass::Fatal,
            429 => HttpRetryClass::RateLimited,
            s if (500..600).contains(&s) => HttpRetryClass::Retry,
            _ => HttpRetryClass::Fatal,
        }
    }

    #[test]
    fn status_100() {
        assert_eq!(classify_http_status(100, None), expected(100));
    }

    #[test]
    fn status_101() {
        assert_eq!(classify_http_status(101, None), expected(101));
    }

    #[test]
    fn status_102() {
        assert_eq!(classify_http_status(102, None), expected(102));
    }

    #[test]
    fn status_103() {
        assert_eq!(classify_http_status(103, None), expected(103));
    }

    #[test]
    fn status_104() {
        assert_eq!(classify_http_status(104, None), expected(104));
    }

    #[test]
    fn status_105() {
        assert_eq!(classify_http_status(105, None), expected(105));
    }

    #[test]
    fn status_106() {
        assert_eq!(classify_http_status(106, None), expected(106));
    }

    #[test]
    fn status_107() {
        assert_eq!(classify_http_status(107, None), expected(107));
    }

    #[test]
    fn status_108() {
        assert_eq!(classify_http_status(108, None), expected(108));
    }

    #[test]
    fn status_109() {
        assert_eq!(classify_http_status(109, None), expected(109));
    }

    #[test]
    fn status_110() {
        assert_eq!(classify_http_status(110, None), expected(110));
    }

    #[test]
    fn status_111() {
        assert_eq!(classify_http_status(111, None), expected(111));
    }

    #[test]
    fn status_112() {
        assert_eq!(classify_http_status(112, None), expected(112));
    }

    #[test]
    fn status_113() {
        assert_eq!(classify_http_status(113, None), expected(113));
    }

    #[test]
    fn status_114() {
        assert_eq!(classify_http_status(114, None), expected(114));
    }

    #[test]
    fn status_115() {
        assert_eq!(classify_http_status(115, None), expected(115));
    }

    #[test]
    fn status_116() {
        assert_eq!(classify_http_status(116, None), expected(116));
    }

    #[test]
    fn status_117() {
        assert_eq!(classify_http_status(117, None), expected(117));
    }

    #[test]
    fn status_118() {
        assert_eq!(classify_http_status(118, None), expected(118));
    }

    #[test]
    fn status_119() {
        assert_eq!(classify_http_status(119, None), expected(119));
    }

    #[test]
    fn status_120() {
        assert_eq!(classify_http_status(120, None), expected(120));
    }

    #[test]
    fn status_121() {
        assert_eq!(classify_http_status(121, None), expected(121));
    }

    #[test]
    fn status_122() {
        assert_eq!(classify_http_status(122, None), expected(122));
    }

    #[test]
    fn status_123() {
        assert_eq!(classify_http_status(123, None), expected(123));
    }

    #[test]
    fn status_124() {
        assert_eq!(classify_http_status(124, None), expected(124));
    }

    #[test]
    fn status_125() {
        assert_eq!(classify_http_status(125, None), expected(125));
    }

    #[test]
    fn status_126() {
        assert_eq!(classify_http_status(126, None), expected(126));
    }

    #[test]
    fn status_127() {
        assert_eq!(classify_http_status(127, None), expected(127));
    }

    #[test]
    fn status_128() {
        assert_eq!(classify_http_status(128, None), expected(128));
    }

    #[test]
    fn status_129() {
        assert_eq!(classify_http_status(129, None), expected(129));
    }

    #[test]
    fn status_130() {
        assert_eq!(classify_http_status(130, None), expected(130));
    }

    #[test]
    fn status_131() {
        assert_eq!(classify_http_status(131, None), expected(131));
    }

    #[test]
    fn status_132() {
        assert_eq!(classify_http_status(132, None), expected(132));
    }

    #[test]
    fn status_133() {
        assert_eq!(classify_http_status(133, None), expected(133));
    }

    #[test]
    fn status_134() {
        assert_eq!(classify_http_status(134, None), expected(134));
    }

    #[test]
    fn status_135() {
        assert_eq!(classify_http_status(135, None), expected(135));
    }

    #[test]
    fn status_136() {
        assert_eq!(classify_http_status(136, None), expected(136));
    }

    #[test]
    fn status_137() {
        assert_eq!(classify_http_status(137, None), expected(137));
    }

    #[test]
    fn status_138() {
        assert_eq!(classify_http_status(138, None), expected(138));
    }

    #[test]
    fn status_139() {
        assert_eq!(classify_http_status(139, None), expected(139));
    }

    #[test]
    fn status_140() {
        assert_eq!(classify_http_status(140, None), expected(140));
    }

    #[test]
    fn status_141() {
        assert_eq!(classify_http_status(141, None), expected(141));
    }

    #[test]
    fn status_142() {
        assert_eq!(classify_http_status(142, None), expected(142));
    }

    #[test]
    fn status_143() {
        assert_eq!(classify_http_status(143, None), expected(143));
    }

    #[test]
    fn status_144() {
        assert_eq!(classify_http_status(144, None), expected(144));
    }

    #[test]
    fn status_145() {
        assert_eq!(classify_http_status(145, None), expected(145));
    }

    #[test]
    fn status_146() {
        assert_eq!(classify_http_status(146, None), expected(146));
    }

    #[test]
    fn status_147() {
        assert_eq!(classify_http_status(147, None), expected(147));
    }

    #[test]
    fn status_148() {
        assert_eq!(classify_http_status(148, None), expected(148));
    }

    #[test]
    fn status_149() {
        assert_eq!(classify_http_status(149, None), expected(149));
    }

    #[test]
    fn status_150() {
        assert_eq!(classify_http_status(150, None), expected(150));
    }

    #[test]
    fn status_151() {
        assert_eq!(classify_http_status(151, None), expected(151));
    }

    #[test]
    fn status_152() {
        assert_eq!(classify_http_status(152, None), expected(152));
    }

    #[test]
    fn status_153() {
        assert_eq!(classify_http_status(153, None), expected(153));
    }

    #[test]
    fn status_154() {
        assert_eq!(classify_http_status(154, None), expected(154));
    }

    #[test]
    fn status_155() {
        assert_eq!(classify_http_status(155, None), expected(155));
    }

    #[test]
    fn status_156() {
        assert_eq!(classify_http_status(156, None), expected(156));
    }

    #[test]
    fn status_157() {
        assert_eq!(classify_http_status(157, None), expected(157));
    }

    #[test]
    fn status_158() {
        assert_eq!(classify_http_status(158, None), expected(158));
    }

    #[test]
    fn status_159() {
        assert_eq!(classify_http_status(159, None), expected(159));
    }

    #[test]
    fn status_160() {
        assert_eq!(classify_http_status(160, None), expected(160));
    }

    #[test]
    fn status_161() {
        assert_eq!(classify_http_status(161, None), expected(161));
    }

    #[test]
    fn status_162() {
        assert_eq!(classify_http_status(162, None), expected(162));
    }

    #[test]
    fn status_163() {
        assert_eq!(classify_http_status(163, None), expected(163));
    }

    #[test]
    fn status_164() {
        assert_eq!(classify_http_status(164, None), expected(164));
    }

    #[test]
    fn status_165() {
        assert_eq!(classify_http_status(165, None), expected(165));
    }

    #[test]
    fn status_166() {
        assert_eq!(classify_http_status(166, None), expected(166));
    }

    #[test]
    fn status_167() {
        assert_eq!(classify_http_status(167, None), expected(167));
    }

    #[test]
    fn status_168() {
        assert_eq!(classify_http_status(168, None), expected(168));
    }

    #[test]
    fn status_169() {
        assert_eq!(classify_http_status(169, None), expected(169));
    }

    #[test]
    fn status_170() {
        assert_eq!(classify_http_status(170, None), expected(170));
    }

    #[test]
    fn status_171() {
        assert_eq!(classify_http_status(171, None), expected(171));
    }

    #[test]
    fn status_172() {
        assert_eq!(classify_http_status(172, None), expected(172));
    }

    #[test]
    fn status_173() {
        assert_eq!(classify_http_status(173, None), expected(173));
    }

    #[test]
    fn status_174() {
        assert_eq!(classify_http_status(174, None), expected(174));
    }

    #[test]
    fn status_175() {
        assert_eq!(classify_http_status(175, None), expected(175));
    }

    #[test]
    fn status_176() {
        assert_eq!(classify_http_status(176, None), expected(176));
    }

    #[test]
    fn status_177() {
        assert_eq!(classify_http_status(177, None), expected(177));
    }

    #[test]
    fn status_178() {
        assert_eq!(classify_http_status(178, None), expected(178));
    }

    #[test]
    fn status_179() {
        assert_eq!(classify_http_status(179, None), expected(179));
    }

    #[test]
    fn status_180() {
        assert_eq!(classify_http_status(180, None), expected(180));
    }

    #[test]
    fn status_181() {
        assert_eq!(classify_http_status(181, None), expected(181));
    }

    #[test]
    fn status_182() {
        assert_eq!(classify_http_status(182, None), expected(182));
    }

    #[test]
    fn status_183() {
        assert_eq!(classify_http_status(183, None), expected(183));
    }

    #[test]
    fn status_184() {
        assert_eq!(classify_http_status(184, None), expected(184));
    }

    #[test]
    fn status_185() {
        assert_eq!(classify_http_status(185, None), expected(185));
    }

    #[test]
    fn status_186() {
        assert_eq!(classify_http_status(186, None), expected(186));
    }

    #[test]
    fn status_187() {
        assert_eq!(classify_http_status(187, None), expected(187));
    }

    #[test]
    fn status_188() {
        assert_eq!(classify_http_status(188, None), expected(188));
    }

    #[test]
    fn status_189() {
        assert_eq!(classify_http_status(189, None), expected(189));
    }

    #[test]
    fn status_190() {
        assert_eq!(classify_http_status(190, None), expected(190));
    }

    #[test]
    fn status_191() {
        assert_eq!(classify_http_status(191, None), expected(191));
    }

    #[test]
    fn status_192() {
        assert_eq!(classify_http_status(192, None), expected(192));
    }

    #[test]
    fn status_193() {
        assert_eq!(classify_http_status(193, None), expected(193));
    }

    #[test]
    fn status_194() {
        assert_eq!(classify_http_status(194, None), expected(194));
    }

    #[test]
    fn status_195() {
        assert_eq!(classify_http_status(195, None), expected(195));
    }

    #[test]
    fn status_196() {
        assert_eq!(classify_http_status(196, None), expected(196));
    }

    #[test]
    fn status_197() {
        assert_eq!(classify_http_status(197, None), expected(197));
    }

    #[test]
    fn status_198() {
        assert_eq!(classify_http_status(198, None), expected(198));
    }

    #[test]
    fn status_199() {
        assert_eq!(classify_http_status(199, None), expected(199));
    }

    #[test]
    fn status_200() {
        assert_eq!(classify_http_status(200, None), expected(200));
    }

    #[test]
    fn status_201() {
        assert_eq!(classify_http_status(201, None), expected(201));
    }

    #[test]
    fn status_202() {
        assert_eq!(classify_http_status(202, None), expected(202));
    }

    #[test]
    fn status_203() {
        assert_eq!(classify_http_status(203, None), expected(203));
    }

    #[test]
    fn status_204() {
        assert_eq!(classify_http_status(204, None), expected(204));
    }

    #[test]
    fn status_205() {
        assert_eq!(classify_http_status(205, None), expected(205));
    }

    #[test]
    fn status_206() {
        assert_eq!(classify_http_status(206, None), expected(206));
    }

    #[test]
    fn status_207() {
        assert_eq!(classify_http_status(207, None), expected(207));
    }

    #[test]
    fn status_208() {
        assert_eq!(classify_http_status(208, None), expected(208));
    }

    #[test]
    fn status_209() {
        assert_eq!(classify_http_status(209, None), expected(209));
    }

    #[test]
    fn status_210() {
        assert_eq!(classify_http_status(210, None), expected(210));
    }

    #[test]
    fn status_211() {
        assert_eq!(classify_http_status(211, None), expected(211));
    }

    #[test]
    fn status_212() {
        assert_eq!(classify_http_status(212, None), expected(212));
    }

    #[test]
    fn status_213() {
        assert_eq!(classify_http_status(213, None), expected(213));
    }

    #[test]
    fn status_214() {
        assert_eq!(classify_http_status(214, None), expected(214));
    }

    #[test]
    fn status_215() {
        assert_eq!(classify_http_status(215, None), expected(215));
    }

    #[test]
    fn status_216() {
        assert_eq!(classify_http_status(216, None), expected(216));
    }

    #[test]
    fn status_217() {
        assert_eq!(classify_http_status(217, None), expected(217));
    }

    #[test]
    fn status_218() {
        assert_eq!(classify_http_status(218, None), expected(218));
    }

    #[test]
    fn status_219() {
        assert_eq!(classify_http_status(219, None), expected(219));
    }

    #[test]
    fn status_220() {
        assert_eq!(classify_http_status(220, None), expected(220));
    }

    #[test]
    fn status_221() {
        assert_eq!(classify_http_status(221, None), expected(221));
    }

    #[test]
    fn status_222() {
        assert_eq!(classify_http_status(222, None), expected(222));
    }

    #[test]
    fn status_223() {
        assert_eq!(classify_http_status(223, None), expected(223));
    }

    #[test]
    fn status_224() {
        assert_eq!(classify_http_status(224, None), expected(224));
    }

    #[test]
    fn status_225() {
        assert_eq!(classify_http_status(225, None), expected(225));
    }

    #[test]
    fn status_226() {
        assert_eq!(classify_http_status(226, None), expected(226));
    }

    #[test]
    fn status_227() {
        assert_eq!(classify_http_status(227, None), expected(227));
    }

    #[test]
    fn status_228() {
        assert_eq!(classify_http_status(228, None), expected(228));
    }

    #[test]
    fn status_229() {
        assert_eq!(classify_http_status(229, None), expected(229));
    }

    #[test]
    fn status_230() {
        assert_eq!(classify_http_status(230, None), expected(230));
    }

    #[test]
    fn status_231() {
        assert_eq!(classify_http_status(231, None), expected(231));
    }

    #[test]
    fn status_232() {
        assert_eq!(classify_http_status(232, None), expected(232));
    }

    #[test]
    fn status_233() {
        assert_eq!(classify_http_status(233, None), expected(233));
    }

    #[test]
    fn status_234() {
        assert_eq!(classify_http_status(234, None), expected(234));
    }

    #[test]
    fn status_235() {
        assert_eq!(classify_http_status(235, None), expected(235));
    }

    #[test]
    fn status_236() {
        assert_eq!(classify_http_status(236, None), expected(236));
    }

    #[test]
    fn status_237() {
        assert_eq!(classify_http_status(237, None), expected(237));
    }

    #[test]
    fn status_238() {
        assert_eq!(classify_http_status(238, None), expected(238));
    }

    #[test]
    fn status_239() {
        assert_eq!(classify_http_status(239, None), expected(239));
    }

    #[test]
    fn status_240() {
        assert_eq!(classify_http_status(240, None), expected(240));
    }

    #[test]
    fn status_241() {
        assert_eq!(classify_http_status(241, None), expected(241));
    }

    #[test]
    fn status_242() {
        assert_eq!(classify_http_status(242, None), expected(242));
    }

    #[test]
    fn status_243() {
        assert_eq!(classify_http_status(243, None), expected(243));
    }

    #[test]
    fn status_244() {
        assert_eq!(classify_http_status(244, None), expected(244));
    }

    #[test]
    fn status_245() {
        assert_eq!(classify_http_status(245, None), expected(245));
    }

    #[test]
    fn status_246() {
        assert_eq!(classify_http_status(246, None), expected(246));
    }

    #[test]
    fn status_247() {
        assert_eq!(classify_http_status(247, None), expected(247));
    }

    #[test]
    fn status_248() {
        assert_eq!(classify_http_status(248, None), expected(248));
    }

    #[test]
    fn status_249() {
        assert_eq!(classify_http_status(249, None), expected(249));
    }

    #[test]
    fn status_250() {
        assert_eq!(classify_http_status(250, None), expected(250));
    }

    #[test]
    fn status_251() {
        assert_eq!(classify_http_status(251, None), expected(251));
    }

    #[test]
    fn status_252() {
        assert_eq!(classify_http_status(252, None), expected(252));
    }

    #[test]
    fn status_253() {
        assert_eq!(classify_http_status(253, None), expected(253));
    }

    #[test]
    fn status_254() {
        assert_eq!(classify_http_status(254, None), expected(254));
    }

    #[test]
    fn status_255() {
        assert_eq!(classify_http_status(255, None), expected(255));
    }

    #[test]
    fn status_256() {
        assert_eq!(classify_http_status(256, None), expected(256));
    }

    #[test]
    fn status_257() {
        assert_eq!(classify_http_status(257, None), expected(257));
    }

    #[test]
    fn status_258() {
        assert_eq!(classify_http_status(258, None), expected(258));
    }

    #[test]
    fn status_259() {
        assert_eq!(classify_http_status(259, None), expected(259));
    }

    #[test]
    fn status_260() {
        assert_eq!(classify_http_status(260, None), expected(260));
    }

    #[test]
    fn status_261() {
        assert_eq!(classify_http_status(261, None), expected(261));
    }

    #[test]
    fn status_262() {
        assert_eq!(classify_http_status(262, None), expected(262));
    }

    #[test]
    fn status_263() {
        assert_eq!(classify_http_status(263, None), expected(263));
    }

    #[test]
    fn status_264() {
        assert_eq!(classify_http_status(264, None), expected(264));
    }

    #[test]
    fn status_265() {
        assert_eq!(classify_http_status(265, None), expected(265));
    }

    #[test]
    fn status_266() {
        assert_eq!(classify_http_status(266, None), expected(266));
    }

    #[test]
    fn status_267() {
        assert_eq!(classify_http_status(267, None), expected(267));
    }

    #[test]
    fn status_268() {
        assert_eq!(classify_http_status(268, None), expected(268));
    }

    #[test]
    fn status_269() {
        assert_eq!(classify_http_status(269, None), expected(269));
    }

    #[test]
    fn status_270() {
        assert_eq!(classify_http_status(270, None), expected(270));
    }

    #[test]
    fn status_271() {
        assert_eq!(classify_http_status(271, None), expected(271));
    }

    #[test]
    fn status_272() {
        assert_eq!(classify_http_status(272, None), expected(272));
    }

    #[test]
    fn status_273() {
        assert_eq!(classify_http_status(273, None), expected(273));
    }

    #[test]
    fn status_274() {
        assert_eq!(classify_http_status(274, None), expected(274));
    }

    #[test]
    fn status_275() {
        assert_eq!(classify_http_status(275, None), expected(275));
    }

    #[test]
    fn status_276() {
        assert_eq!(classify_http_status(276, None), expected(276));
    }

    #[test]
    fn status_277() {
        assert_eq!(classify_http_status(277, None), expected(277));
    }

    #[test]
    fn status_278() {
        assert_eq!(classify_http_status(278, None), expected(278));
    }

    #[test]
    fn status_279() {
        assert_eq!(classify_http_status(279, None), expected(279));
    }

    #[test]
    fn status_280() {
        assert_eq!(classify_http_status(280, None), expected(280));
    }

    #[test]
    fn status_281() {
        assert_eq!(classify_http_status(281, None), expected(281));
    }

    #[test]
    fn status_282() {
        assert_eq!(classify_http_status(282, None), expected(282));
    }

    #[test]
    fn status_283() {
        assert_eq!(classify_http_status(283, None), expected(283));
    }

    #[test]
    fn status_284() {
        assert_eq!(classify_http_status(284, None), expected(284));
    }

    #[test]
    fn status_285() {
        assert_eq!(classify_http_status(285, None), expected(285));
    }

    #[test]
    fn status_286() {
        assert_eq!(classify_http_status(286, None), expected(286));
    }

    #[test]
    fn status_287() {
        assert_eq!(classify_http_status(287, None), expected(287));
    }

    #[test]
    fn status_288() {
        assert_eq!(classify_http_status(288, None), expected(288));
    }

    #[test]
    fn status_289() {
        assert_eq!(classify_http_status(289, None), expected(289));
    }

    #[test]
    fn status_290() {
        assert_eq!(classify_http_status(290, None), expected(290));
    }

    #[test]
    fn status_291() {
        assert_eq!(classify_http_status(291, None), expected(291));
    }

    #[test]
    fn status_292() {
        assert_eq!(classify_http_status(292, None), expected(292));
    }

    #[test]
    fn status_293() {
        assert_eq!(classify_http_status(293, None), expected(293));
    }

    #[test]
    fn status_294() {
        assert_eq!(classify_http_status(294, None), expected(294));
    }

    #[test]
    fn status_295() {
        assert_eq!(classify_http_status(295, None), expected(295));
    }

    #[test]
    fn status_296() {
        assert_eq!(classify_http_status(296, None), expected(296));
    }

    #[test]
    fn status_297() {
        assert_eq!(classify_http_status(297, None), expected(297));
    }

    #[test]
    fn status_298() {
        assert_eq!(classify_http_status(298, None), expected(298));
    }

    #[test]
    fn status_299() {
        assert_eq!(classify_http_status(299, None), expected(299));
    }

    #[test]
    fn status_300() {
        assert_eq!(classify_http_status(300, None), expected(300));
    }

    #[test]
    fn status_301() {
        assert_eq!(classify_http_status(301, None), expected(301));
    }

    #[test]
    fn status_302() {
        assert_eq!(classify_http_status(302, None), expected(302));
    }

    #[test]
    fn status_303() {
        assert_eq!(classify_http_status(303, None), expected(303));
    }

    #[test]
    fn status_304() {
        assert_eq!(classify_http_status(304, None), expected(304));
    }

    #[test]
    fn status_305() {
        assert_eq!(classify_http_status(305, None), expected(305));
    }

    #[test]
    fn status_306() {
        assert_eq!(classify_http_status(306, None), expected(306));
    }

    #[test]
    fn status_307() {
        assert_eq!(classify_http_status(307, None), expected(307));
    }

    #[test]
    fn status_308() {
        assert_eq!(classify_http_status(308, None), expected(308));
    }

    #[test]
    fn status_309() {
        assert_eq!(classify_http_status(309, None), expected(309));
    }

    #[test]
    fn status_310() {
        assert_eq!(classify_http_status(310, None), expected(310));
    }

    #[test]
    fn status_311() {
        assert_eq!(classify_http_status(311, None), expected(311));
    }

    #[test]
    fn status_312() {
        assert_eq!(classify_http_status(312, None), expected(312));
    }

    #[test]
    fn status_313() {
        assert_eq!(classify_http_status(313, None), expected(313));
    }

    #[test]
    fn status_314() {
        assert_eq!(classify_http_status(314, None), expected(314));
    }

    #[test]
    fn status_315() {
        assert_eq!(classify_http_status(315, None), expected(315));
    }

    #[test]
    fn status_316() {
        assert_eq!(classify_http_status(316, None), expected(316));
    }

    #[test]
    fn status_317() {
        assert_eq!(classify_http_status(317, None), expected(317));
    }

    #[test]
    fn status_318() {
        assert_eq!(classify_http_status(318, None), expected(318));
    }

    #[test]
    fn status_319() {
        assert_eq!(classify_http_status(319, None), expected(319));
    }

    #[test]
    fn status_320() {
        assert_eq!(classify_http_status(320, None), expected(320));
    }

    #[test]
    fn status_321() {
        assert_eq!(classify_http_status(321, None), expected(321));
    }

    #[test]
    fn status_322() {
        assert_eq!(classify_http_status(322, None), expected(322));
    }

    #[test]
    fn status_323() {
        assert_eq!(classify_http_status(323, None), expected(323));
    }

    #[test]
    fn status_324() {
        assert_eq!(classify_http_status(324, None), expected(324));
    }

    #[test]
    fn status_325() {
        assert_eq!(classify_http_status(325, None), expected(325));
    }

    #[test]
    fn status_326() {
        assert_eq!(classify_http_status(326, None), expected(326));
    }

    #[test]
    fn status_327() {
        assert_eq!(classify_http_status(327, None), expected(327));
    }

    #[test]
    fn status_328() {
        assert_eq!(classify_http_status(328, None), expected(328));
    }

    #[test]
    fn status_329() {
        assert_eq!(classify_http_status(329, None), expected(329));
    }

    #[test]
    fn status_330() {
        assert_eq!(classify_http_status(330, None), expected(330));
    }

    #[test]
    fn status_331() {
        assert_eq!(classify_http_status(331, None), expected(331));
    }

    #[test]
    fn status_332() {
        assert_eq!(classify_http_status(332, None), expected(332));
    }

    #[test]
    fn status_333() {
        assert_eq!(classify_http_status(333, None), expected(333));
    }

    #[test]
    fn status_334() {
        assert_eq!(classify_http_status(334, None), expected(334));
    }

    #[test]
    fn status_335() {
        assert_eq!(classify_http_status(335, None), expected(335));
    }

    #[test]
    fn status_336() {
        assert_eq!(classify_http_status(336, None), expected(336));
    }

    #[test]
    fn status_337() {
        assert_eq!(classify_http_status(337, None), expected(337));
    }

    #[test]
    fn status_338() {
        assert_eq!(classify_http_status(338, None), expected(338));
    }

    #[test]
    fn status_339() {
        assert_eq!(classify_http_status(339, None), expected(339));
    }

    #[test]
    fn status_340() {
        assert_eq!(classify_http_status(340, None), expected(340));
    }

    #[test]
    fn status_341() {
        assert_eq!(classify_http_status(341, None), expected(341));
    }

    #[test]
    fn status_342() {
        assert_eq!(classify_http_status(342, None), expected(342));
    }

    #[test]
    fn status_343() {
        assert_eq!(classify_http_status(343, None), expected(343));
    }

    #[test]
    fn status_344() {
        assert_eq!(classify_http_status(344, None), expected(344));
    }

    #[test]
    fn status_345() {
        assert_eq!(classify_http_status(345, None), expected(345));
    }

    #[test]
    fn status_346() {
        assert_eq!(classify_http_status(346, None), expected(346));
    }

    #[test]
    fn status_347() {
        assert_eq!(classify_http_status(347, None), expected(347));
    }

    #[test]
    fn status_348() {
        assert_eq!(classify_http_status(348, None), expected(348));
    }

    #[test]
    fn status_349() {
        assert_eq!(classify_http_status(349, None), expected(349));
    }

    #[test]
    fn status_350() {
        assert_eq!(classify_http_status(350, None), expected(350));
    }

    #[test]
    fn status_351() {
        assert_eq!(classify_http_status(351, None), expected(351));
    }

    #[test]
    fn status_352() {
        assert_eq!(classify_http_status(352, None), expected(352));
    }

    #[test]
    fn status_353() {
        assert_eq!(classify_http_status(353, None), expected(353));
    }

    #[test]
    fn status_354() {
        assert_eq!(classify_http_status(354, None), expected(354));
    }

    #[test]
    fn status_355() {
        assert_eq!(classify_http_status(355, None), expected(355));
    }

    #[test]
    fn status_356() {
        assert_eq!(classify_http_status(356, None), expected(356));
    }

    #[test]
    fn status_357() {
        assert_eq!(classify_http_status(357, None), expected(357));
    }

    #[test]
    fn status_358() {
        assert_eq!(classify_http_status(358, None), expected(358));
    }

    #[test]
    fn status_359() {
        assert_eq!(classify_http_status(359, None), expected(359));
    }

    #[test]
    fn status_360() {
        assert_eq!(classify_http_status(360, None), expected(360));
    }

    #[test]
    fn status_361() {
        assert_eq!(classify_http_status(361, None), expected(361));
    }

    #[test]
    fn status_362() {
        assert_eq!(classify_http_status(362, None), expected(362));
    }

    #[test]
    fn status_363() {
        assert_eq!(classify_http_status(363, None), expected(363));
    }

    #[test]
    fn status_364() {
        assert_eq!(classify_http_status(364, None), expected(364));
    }

    #[test]
    fn status_365() {
        assert_eq!(classify_http_status(365, None), expected(365));
    }

    #[test]
    fn status_366() {
        assert_eq!(classify_http_status(366, None), expected(366));
    }

    #[test]
    fn status_367() {
        assert_eq!(classify_http_status(367, None), expected(367));
    }

    #[test]
    fn status_368() {
        assert_eq!(classify_http_status(368, None), expected(368));
    }

    #[test]
    fn status_369() {
        assert_eq!(classify_http_status(369, None), expected(369));
    }

    #[test]
    fn status_370() {
        assert_eq!(classify_http_status(370, None), expected(370));
    }

    #[test]
    fn status_371() {
        assert_eq!(classify_http_status(371, None), expected(371));
    }

    #[test]
    fn status_372() {
        assert_eq!(classify_http_status(372, None), expected(372));
    }

    #[test]
    fn status_373() {
        assert_eq!(classify_http_status(373, None), expected(373));
    }

    #[test]
    fn status_374() {
        assert_eq!(classify_http_status(374, None), expected(374));
    }

    #[test]
    fn status_375() {
        assert_eq!(classify_http_status(375, None), expected(375));
    }

    #[test]
    fn status_376() {
        assert_eq!(classify_http_status(376, None), expected(376));
    }

    #[test]
    fn status_377() {
        assert_eq!(classify_http_status(377, None), expected(377));
    }

    #[test]
    fn status_378() {
        assert_eq!(classify_http_status(378, None), expected(378));
    }

    #[test]
    fn status_379() {
        assert_eq!(classify_http_status(379, None), expected(379));
    }

    #[test]
    fn status_380() {
        assert_eq!(classify_http_status(380, None), expected(380));
    }

    #[test]
    fn status_381() {
        assert_eq!(classify_http_status(381, None), expected(381));
    }

    #[test]
    fn status_382() {
        assert_eq!(classify_http_status(382, None), expected(382));
    }

    #[test]
    fn status_383() {
        assert_eq!(classify_http_status(383, None), expected(383));
    }

    #[test]
    fn status_384() {
        assert_eq!(classify_http_status(384, None), expected(384));
    }

    #[test]
    fn status_385() {
        assert_eq!(classify_http_status(385, None), expected(385));
    }

    #[test]
    fn status_386() {
        assert_eq!(classify_http_status(386, None), expected(386));
    }

    #[test]
    fn status_387() {
        assert_eq!(classify_http_status(387, None), expected(387));
    }

    #[test]
    fn status_388() {
        assert_eq!(classify_http_status(388, None), expected(388));
    }

    #[test]
    fn status_389() {
        assert_eq!(classify_http_status(389, None), expected(389));
    }

    #[test]
    fn status_390() {
        assert_eq!(classify_http_status(390, None), expected(390));
    }

    #[test]
    fn status_391() {
        assert_eq!(classify_http_status(391, None), expected(391));
    }

    #[test]
    fn status_392() {
        assert_eq!(classify_http_status(392, None), expected(392));
    }

    #[test]
    fn status_393() {
        assert_eq!(classify_http_status(393, None), expected(393));
    }

    #[test]
    fn status_394() {
        assert_eq!(classify_http_status(394, None), expected(394));
    }

    #[test]
    fn status_395() {
        assert_eq!(classify_http_status(395, None), expected(395));
    }

    #[test]
    fn status_396() {
        assert_eq!(classify_http_status(396, None), expected(396));
    }

    #[test]
    fn status_397() {
        assert_eq!(classify_http_status(397, None), expected(397));
    }

    #[test]
    fn status_398() {
        assert_eq!(classify_http_status(398, None), expected(398));
    }

    #[test]
    fn status_399() {
        assert_eq!(classify_http_status(399, None), expected(399));
    }

    #[test]
    fn status_400() {
        assert_eq!(classify_http_status(400, None), expected(400));
    }

    #[test]
    fn status_401() {
        assert_eq!(classify_http_status(401, None), expected(401));
    }

    #[test]
    fn status_402() {
        assert_eq!(classify_http_status(402, None), expected(402));
    }

    #[test]
    fn status_403() {
        assert_eq!(classify_http_status(403, None), expected(403));
    }

    #[test]
    fn status_404() {
        assert_eq!(classify_http_status(404, None), expected(404));
    }

    #[test]
    fn status_405() {
        assert_eq!(classify_http_status(405, None), expected(405));
    }

    #[test]
    fn status_406() {
        assert_eq!(classify_http_status(406, None), expected(406));
    }

    #[test]
    fn status_407() {
        assert_eq!(classify_http_status(407, None), expected(407));
    }

    #[test]
    fn status_408() {
        assert_eq!(classify_http_status(408, None), expected(408));
    }

    #[test]
    fn status_409() {
        assert_eq!(classify_http_status(409, None), expected(409));
    }

    #[test]
    fn status_410() {
        assert_eq!(classify_http_status(410, None), expected(410));
    }

    #[test]
    fn status_411() {
        assert_eq!(classify_http_status(411, None), expected(411));
    }

    #[test]
    fn status_412() {
        assert_eq!(classify_http_status(412, None), expected(412));
    }

    #[test]
    fn status_413() {
        assert_eq!(classify_http_status(413, None), expected(413));
    }

    #[test]
    fn status_414() {
        assert_eq!(classify_http_status(414, None), expected(414));
    }

    #[test]
    fn status_415() {
        assert_eq!(classify_http_status(415, None), expected(415));
    }

    #[test]
    fn status_416() {
        assert_eq!(classify_http_status(416, None), expected(416));
    }

    #[test]
    fn status_417() {
        assert_eq!(classify_http_status(417, None), expected(417));
    }

    #[test]
    fn status_418() {
        assert_eq!(classify_http_status(418, None), expected(418));
    }

    #[test]
    fn status_419() {
        assert_eq!(classify_http_status(419, None), expected(419));
    }

    #[test]
    fn status_420() {
        assert_eq!(classify_http_status(420, None), expected(420));
    }

    #[test]
    fn status_421() {
        assert_eq!(classify_http_status(421, None), expected(421));
    }

    #[test]
    fn status_422() {
        assert_eq!(classify_http_status(422, None), expected(422));
    }

    #[test]
    fn status_423() {
        assert_eq!(classify_http_status(423, None), expected(423));
    }

    #[test]
    fn status_424() {
        assert_eq!(classify_http_status(424, None), expected(424));
    }

    #[test]
    fn status_425() {
        assert_eq!(classify_http_status(425, None), expected(425));
    }

    #[test]
    fn status_426() {
        assert_eq!(classify_http_status(426, None), expected(426));
    }

    #[test]
    fn status_427() {
        assert_eq!(classify_http_status(427, None), expected(427));
    }

    #[test]
    fn status_428() {
        assert_eq!(classify_http_status(428, None), expected(428));
    }

    #[test]
    fn status_429() {
        assert_eq!(classify_http_status(429, None), expected(429));
    }

    #[test]
    fn status_430() {
        assert_eq!(classify_http_status(430, None), expected(430));
    }

    #[test]
    fn status_431() {
        assert_eq!(classify_http_status(431, None), expected(431));
    }

    #[test]
    fn status_432() {
        assert_eq!(classify_http_status(432, None), expected(432));
    }

    #[test]
    fn status_433() {
        assert_eq!(classify_http_status(433, None), expected(433));
    }

    #[test]
    fn status_434() {
        assert_eq!(classify_http_status(434, None), expected(434));
    }

    #[test]
    fn status_435() {
        assert_eq!(classify_http_status(435, None), expected(435));
    }

    #[test]
    fn status_436() {
        assert_eq!(classify_http_status(436, None), expected(436));
    }

    #[test]
    fn status_437() {
        assert_eq!(classify_http_status(437, None), expected(437));
    }

    #[test]
    fn status_438() {
        assert_eq!(classify_http_status(438, None), expected(438));
    }

    #[test]
    fn status_439() {
        assert_eq!(classify_http_status(439, None), expected(439));
    }

    #[test]
    fn status_440() {
        assert_eq!(classify_http_status(440, None), expected(440));
    }

    #[test]
    fn status_441() {
        assert_eq!(classify_http_status(441, None), expected(441));
    }

    #[test]
    fn status_442() {
        assert_eq!(classify_http_status(442, None), expected(442));
    }

    #[test]
    fn status_443() {
        assert_eq!(classify_http_status(443, None), expected(443));
    }

    #[test]
    fn status_444() {
        assert_eq!(classify_http_status(444, None), expected(444));
    }

    #[test]
    fn status_445() {
        assert_eq!(classify_http_status(445, None), expected(445));
    }

    #[test]
    fn status_446() {
        assert_eq!(classify_http_status(446, None), expected(446));
    }

    #[test]
    fn status_447() {
        assert_eq!(classify_http_status(447, None), expected(447));
    }

    #[test]
    fn status_448() {
        assert_eq!(classify_http_status(448, None), expected(448));
    }

    #[test]
    fn status_449() {
        assert_eq!(classify_http_status(449, None), expected(449));
    }

    #[test]
    fn status_450() {
        assert_eq!(classify_http_status(450, None), expected(450));
    }

    #[test]
    fn status_451() {
        assert_eq!(classify_http_status(451, None), expected(451));
    }

    #[test]
    fn status_452() {
        assert_eq!(classify_http_status(452, None), expected(452));
    }

    #[test]
    fn status_453() {
        assert_eq!(classify_http_status(453, None), expected(453));
    }

    #[test]
    fn status_454() {
        assert_eq!(classify_http_status(454, None), expected(454));
    }

    #[test]
    fn status_455() {
        assert_eq!(classify_http_status(455, None), expected(455));
    }

    #[test]
    fn status_456() {
        assert_eq!(classify_http_status(456, None), expected(456));
    }

    #[test]
    fn status_457() {
        assert_eq!(classify_http_status(457, None), expected(457));
    }

    #[test]
    fn status_458() {
        assert_eq!(classify_http_status(458, None), expected(458));
    }

    #[test]
    fn status_459() {
        assert_eq!(classify_http_status(459, None), expected(459));
    }

    #[test]
    fn status_460() {
        assert_eq!(classify_http_status(460, None), expected(460));
    }

    #[test]
    fn status_461() {
        assert_eq!(classify_http_status(461, None), expected(461));
    }

    #[test]
    fn status_462() {
        assert_eq!(classify_http_status(462, None), expected(462));
    }

    #[test]
    fn status_463() {
        assert_eq!(classify_http_status(463, None), expected(463));
    }

    #[test]
    fn status_464() {
        assert_eq!(classify_http_status(464, None), expected(464));
    }

    #[test]
    fn status_465() {
        assert_eq!(classify_http_status(465, None), expected(465));
    }

    #[test]
    fn status_466() {
        assert_eq!(classify_http_status(466, None), expected(466));
    }

    #[test]
    fn status_467() {
        assert_eq!(classify_http_status(467, None), expected(467));
    }

    #[test]
    fn status_468() {
        assert_eq!(classify_http_status(468, None), expected(468));
    }

    #[test]
    fn status_469() {
        assert_eq!(classify_http_status(469, None), expected(469));
    }

    #[test]
    fn status_470() {
        assert_eq!(classify_http_status(470, None), expected(470));
    }

    #[test]
    fn status_471() {
        assert_eq!(classify_http_status(471, None), expected(471));
    }

    #[test]
    fn status_472() {
        assert_eq!(classify_http_status(472, None), expected(472));
    }

    #[test]
    fn status_473() {
        assert_eq!(classify_http_status(473, None), expected(473));
    }

    #[test]
    fn status_474() {
        assert_eq!(classify_http_status(474, None), expected(474));
    }

    #[test]
    fn status_475() {
        assert_eq!(classify_http_status(475, None), expected(475));
    }

    #[test]
    fn status_476() {
        assert_eq!(classify_http_status(476, None), expected(476));
    }

    #[test]
    fn status_477() {
        assert_eq!(classify_http_status(477, None), expected(477));
    }

    #[test]
    fn status_478() {
        assert_eq!(classify_http_status(478, None), expected(478));
    }

    #[test]
    fn status_479() {
        assert_eq!(classify_http_status(479, None), expected(479));
    }

    #[test]
    fn status_480() {
        assert_eq!(classify_http_status(480, None), expected(480));
    }

    #[test]
    fn status_481() {
        assert_eq!(classify_http_status(481, None), expected(481));
    }

    #[test]
    fn status_482() {
        assert_eq!(classify_http_status(482, None), expected(482));
    }

    #[test]
    fn status_483() {
        assert_eq!(classify_http_status(483, None), expected(483));
    }

    #[test]
    fn status_484() {
        assert_eq!(classify_http_status(484, None), expected(484));
    }

    #[test]
    fn status_485() {
        assert_eq!(classify_http_status(485, None), expected(485));
    }

    #[test]
    fn status_486() {
        assert_eq!(classify_http_status(486, None), expected(486));
    }

    #[test]
    fn status_487() {
        assert_eq!(classify_http_status(487, None), expected(487));
    }

    #[test]
    fn status_488() {
        assert_eq!(classify_http_status(488, None), expected(488));
    }

    #[test]
    fn status_489() {
        assert_eq!(classify_http_status(489, None), expected(489));
    }

    #[test]
    fn status_490() {
        assert_eq!(classify_http_status(490, None), expected(490));
    }

    #[test]
    fn status_491() {
        assert_eq!(classify_http_status(491, None), expected(491));
    }

    #[test]
    fn status_492() {
        assert_eq!(classify_http_status(492, None), expected(492));
    }

    #[test]
    fn status_493() {
        assert_eq!(classify_http_status(493, None), expected(493));
    }

    #[test]
    fn status_494() {
        assert_eq!(classify_http_status(494, None), expected(494));
    }

    #[test]
    fn status_495() {
        assert_eq!(classify_http_status(495, None), expected(495));
    }

    #[test]
    fn status_496() {
        assert_eq!(classify_http_status(496, None), expected(496));
    }

    #[test]
    fn status_497() {
        assert_eq!(classify_http_status(497, None), expected(497));
    }

    #[test]
    fn status_498() {
        assert_eq!(classify_http_status(498, None), expected(498));
    }

    #[test]
    fn status_499() {
        assert_eq!(classify_http_status(499, None), expected(499));
    }

    #[test]
    fn status_500() {
        assert_eq!(classify_http_status(500, None), expected(500));
    }

    #[test]
    fn status_501() {
        assert_eq!(classify_http_status(501, None), expected(501));
    }

    #[test]
    fn status_502() {
        assert_eq!(classify_http_status(502, None), expected(502));
    }

    #[test]
    fn status_503() {
        assert_eq!(classify_http_status(503, None), expected(503));
    }

    #[test]
    fn status_504() {
        assert_eq!(classify_http_status(504, None), expected(504));
    }

    #[test]
    fn status_505() {
        assert_eq!(classify_http_status(505, None), expected(505));
    }

    #[test]
    fn status_506() {
        assert_eq!(classify_http_status(506, None), expected(506));
    }

    #[test]
    fn status_507() {
        assert_eq!(classify_http_status(507, None), expected(507));
    }

    #[test]
    fn status_508() {
        assert_eq!(classify_http_status(508, None), expected(508));
    }

    #[test]
    fn status_509() {
        assert_eq!(classify_http_status(509, None), expected(509));
    }

    #[test]
    fn status_510() {
        assert_eq!(classify_http_status(510, None), expected(510));
    }

    #[test]
    fn status_511() {
        assert_eq!(classify_http_status(511, None), expected(511));
    }

    #[test]
    fn status_512() {
        assert_eq!(classify_http_status(512, None), expected(512));
    }

    #[test]
    fn status_513() {
        assert_eq!(classify_http_status(513, None), expected(513));
    }

    #[test]
    fn status_514() {
        assert_eq!(classify_http_status(514, None), expected(514));
    }

    #[test]
    fn status_515() {
        assert_eq!(classify_http_status(515, None), expected(515));
    }

    #[test]
    fn status_516() {
        assert_eq!(classify_http_status(516, None), expected(516));
    }

    #[test]
    fn status_517() {
        assert_eq!(classify_http_status(517, None), expected(517));
    }

    #[test]
    fn status_518() {
        assert_eq!(classify_http_status(518, None), expected(518));
    }

    #[test]
    fn status_519() {
        assert_eq!(classify_http_status(519, None), expected(519));
    }

    #[test]
    fn status_520() {
        assert_eq!(classify_http_status(520, None), expected(520));
    }

    #[test]
    fn status_521() {
        assert_eq!(classify_http_status(521, None), expected(521));
    }

    #[test]
    fn status_522() {
        assert_eq!(classify_http_status(522, None), expected(522));
    }

    #[test]
    fn status_523() {
        assert_eq!(classify_http_status(523, None), expected(523));
    }

    #[test]
    fn status_524() {
        assert_eq!(classify_http_status(524, None), expected(524));
    }

    #[test]
    fn status_525() {
        assert_eq!(classify_http_status(525, None), expected(525));
    }

    #[test]
    fn status_526() {
        assert_eq!(classify_http_status(526, None), expected(526));
    }

    #[test]
    fn status_527() {
        assert_eq!(classify_http_status(527, None), expected(527));
    }

    #[test]
    fn status_528() {
        assert_eq!(classify_http_status(528, None), expected(528));
    }

    #[test]
    fn status_529() {
        assert_eq!(classify_http_status(529, None), expected(529));
    }

    #[test]
    fn status_530() {
        assert_eq!(classify_http_status(530, None), expected(530));
    }

    #[test]
    fn status_531() {
        assert_eq!(classify_http_status(531, None), expected(531));
    }

    #[test]
    fn status_532() {
        assert_eq!(classify_http_status(532, None), expected(532));
    }

    #[test]
    fn status_533() {
        assert_eq!(classify_http_status(533, None), expected(533));
    }

    #[test]
    fn status_534() {
        assert_eq!(classify_http_status(534, None), expected(534));
    }

    #[test]
    fn status_535() {
        assert_eq!(classify_http_status(535, None), expected(535));
    }

    #[test]
    fn status_536() {
        assert_eq!(classify_http_status(536, None), expected(536));
    }

    #[test]
    fn status_537() {
        assert_eq!(classify_http_status(537, None), expected(537));
    }

    #[test]
    fn status_538() {
        assert_eq!(classify_http_status(538, None), expected(538));
    }

    #[test]
    fn status_539() {
        assert_eq!(classify_http_status(539, None), expected(539));
    }

    #[test]
    fn status_540() {
        assert_eq!(classify_http_status(540, None), expected(540));
    }

    #[test]
    fn status_541() {
        assert_eq!(classify_http_status(541, None), expected(541));
    }

    #[test]
    fn status_542() {
        assert_eq!(classify_http_status(542, None), expected(542));
    }

    #[test]
    fn status_543() {
        assert_eq!(classify_http_status(543, None), expected(543));
    }

    #[test]
    fn status_544() {
        assert_eq!(classify_http_status(544, None), expected(544));
    }

    #[test]
    fn status_545() {
        assert_eq!(classify_http_status(545, None), expected(545));
    }

    #[test]
    fn status_546() {
        assert_eq!(classify_http_status(546, None), expected(546));
    }

    #[test]
    fn status_547() {
        assert_eq!(classify_http_status(547, None), expected(547));
    }

    #[test]
    fn status_548() {
        assert_eq!(classify_http_status(548, None), expected(548));
    }

    #[test]
    fn status_549() {
        assert_eq!(classify_http_status(549, None), expected(549));
    }

    #[test]
    fn status_550() {
        assert_eq!(classify_http_status(550, None), expected(550));
    }

    #[test]
    fn status_551() {
        assert_eq!(classify_http_status(551, None), expected(551));
    }

    #[test]
    fn status_552() {
        assert_eq!(classify_http_status(552, None), expected(552));
    }

    #[test]
    fn status_553() {
        assert_eq!(classify_http_status(553, None), expected(553));
    }

    #[test]
    fn status_554() {
        assert_eq!(classify_http_status(554, None), expected(554));
    }

    #[test]
    fn status_555() {
        assert_eq!(classify_http_status(555, None), expected(555));
    }

    #[test]
    fn status_556() {
        assert_eq!(classify_http_status(556, None), expected(556));
    }

    #[test]
    fn status_557() {
        assert_eq!(classify_http_status(557, None), expected(557));
    }

    #[test]
    fn status_558() {
        assert_eq!(classify_http_status(558, None), expected(558));
    }

    #[test]
    fn status_559() {
        assert_eq!(classify_http_status(559, None), expected(559));
    }

    #[test]
    fn status_560() {
        assert_eq!(classify_http_status(560, None), expected(560));
    }

    #[test]
    fn status_561() {
        assert_eq!(classify_http_status(561, None), expected(561));
    }

    #[test]
    fn status_562() {
        assert_eq!(classify_http_status(562, None), expected(562));
    }

    #[test]
    fn status_563() {
        assert_eq!(classify_http_status(563, None), expected(563));
    }

    #[test]
    fn status_564() {
        assert_eq!(classify_http_status(564, None), expected(564));
    }

    #[test]
    fn status_565() {
        assert_eq!(classify_http_status(565, None), expected(565));
    }

    #[test]
    fn status_566() {
        assert_eq!(classify_http_status(566, None), expected(566));
    }

    #[test]
    fn status_567() {
        assert_eq!(classify_http_status(567, None), expected(567));
    }

    #[test]
    fn status_568() {
        assert_eq!(classify_http_status(568, None), expected(568));
    }

    #[test]
    fn status_569() {
        assert_eq!(classify_http_status(569, None), expected(569));
    }

    #[test]
    fn status_570() {
        assert_eq!(classify_http_status(570, None), expected(570));
    }

    #[test]
    fn status_571() {
        assert_eq!(classify_http_status(571, None), expected(571));
    }

    #[test]
    fn status_572() {
        assert_eq!(classify_http_status(572, None), expected(572));
    }

    #[test]
    fn status_573() {
        assert_eq!(classify_http_status(573, None), expected(573));
    }

    #[test]
    fn status_574() {
        assert_eq!(classify_http_status(574, None), expected(574));
    }

    #[test]
    fn status_575() {
        assert_eq!(classify_http_status(575, None), expected(575));
    }

    #[test]
    fn status_576() {
        assert_eq!(classify_http_status(576, None), expected(576));
    }

    #[test]
    fn status_577() {
        assert_eq!(classify_http_status(577, None), expected(577));
    }

    #[test]
    fn status_578() {
        assert_eq!(classify_http_status(578, None), expected(578));
    }

    #[test]
    fn status_579() {
        assert_eq!(classify_http_status(579, None), expected(579));
    }

    #[test]
    fn status_580() {
        assert_eq!(classify_http_status(580, None), expected(580));
    }

    #[test]
    fn status_581() {
        assert_eq!(classify_http_status(581, None), expected(581));
    }

    #[test]
    fn status_582() {
        assert_eq!(classify_http_status(582, None), expected(582));
    }

    #[test]
    fn status_583() {
        assert_eq!(classify_http_status(583, None), expected(583));
    }

    #[test]
    fn status_584() {
        assert_eq!(classify_http_status(584, None), expected(584));
    }

    #[test]
    fn status_585() {
        assert_eq!(classify_http_status(585, None), expected(585));
    }

    #[test]
    fn status_586() {
        assert_eq!(classify_http_status(586, None), expected(586));
    }

    #[test]
    fn status_587() {
        assert_eq!(classify_http_status(587, None), expected(587));
    }

    #[test]
    fn status_588() {
        assert_eq!(classify_http_status(588, None), expected(588));
    }

    #[test]
    fn status_589() {
        assert_eq!(classify_http_status(589, None), expected(589));
    }

    #[test]
    fn status_590() {
        assert_eq!(classify_http_status(590, None), expected(590));
    }

    #[test]
    fn status_591() {
        assert_eq!(classify_http_status(591, None), expected(591));
    }

    #[test]
    fn status_592() {
        assert_eq!(classify_http_status(592, None), expected(592));
    }

    #[test]
    fn status_593() {
        assert_eq!(classify_http_status(593, None), expected(593));
    }

    #[test]
    fn status_594() {
        assert_eq!(classify_http_status(594, None), expected(594));
    }

    #[test]
    fn status_595() {
        assert_eq!(classify_http_status(595, None), expected(595));
    }

    #[test]
    fn status_596() {
        assert_eq!(classify_http_status(596, None), expected(596));
    }

    #[test]
    fn status_597() {
        assert_eq!(classify_http_status(597, None), expected(597));
    }

    #[test]
    fn status_598() {
        assert_eq!(classify_http_status(598, None), expected(598));
    }

    #[test]
    fn status_599() {
        assert_eq!(classify_http_status(599, None), expected(599));
    }

    #[test]
    fn status_200_x_should_retry_true() {
        assert_eq!(classify_http_status(200, Some(true)), HttpRetryClass::Retry);
    }
    #[test]
    fn status_200_x_should_retry_false() {
        assert_eq!(classify_http_status(200, Some(false)), HttpRetryClass::Fatal);
    }

    #[test]
    fn status_418_x_should_retry_true() {
        assert_eq!(classify_http_status(418, Some(true)), HttpRetryClass::Retry);
    }
    #[test]
    fn status_418_x_should_retry_false() {
        assert_eq!(classify_http_status(418, Some(false)), HttpRetryClass::Fatal);
    }

    #[test]
    fn status_450_x_should_retry_true() {
        assert_eq!(classify_http_status(450, Some(true)), HttpRetryClass::Retry);
    }
    #[test]
    fn status_450_x_should_retry_false() {
        assert_eq!(classify_http_status(450, Some(false)), HttpRetryClass::Fatal);
    }

    #[test]
    fn status_499_x_should_retry_true() {
        assert_eq!(classify_http_status(499, Some(true)), HttpRetryClass::Retry);
    }
    #[test]
    fn status_499_x_should_retry_false() {
        assert_eq!(classify_http_status(499, Some(false)), HttpRetryClass::Fatal);
    }
}
